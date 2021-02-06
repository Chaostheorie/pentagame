use super::errors::WebsocketError;
use super::messages::{
    Connect, Disconnect, MakeMoveMessage, QueryGameMessage, QueryMovesMessage, SessionMessage,
    StartGameMessage,
};
use crate::frontend::helper::log_error;
use crate::graph::{graph::GraphState, graph::ResizableGraphState, graph::GRAPH};
use crate::state::GameServerState;
use actix::prelude::*;
use cached::stores::TimedCache;
use dashmap::{DashMap, DashSet};
use rayon::prelude::*;
use std::convert::TryInto;
use std::sync::Arc;
use uuid::Uuid;

// `GameServer` manages  and responsible for coordinating game sessions
pub struct GameServer {
    state: Arc<GameServerState>,
    conn: Mutex<PooledConnection>,
}

impl GameServer {
    // Send message to all users in the room
    fn send_message(&self, game: &i32, action: u8, data: DashMap<String, String>) {
        if let Some(sessions) = self.0.sessions.get(game) {
            // sweet parallel overkill
            sessions.value().into_par_iter().for_each(|id| {
                let _ = id.do_send(SessionMessage {
                    action,
                    data: data.clone(),
                });
            })
        }
    }
}

// Make actor from `GameServer`
impl Actor for GameServer {
    // We are going to use simple Context, we just need ability to communicate
    // with other actors.
    type Context = Context<Self>;
}

// Handler for Connect message.
//
// Register new session and assign unique id to this session
impl Handler<Connect> for GameServer {
    type Result = Result<(), WebsocketError>;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // register session with random id. The +1 ensures that 0 is never a session id
        // to enable 0 as placeholder for nobody when skipping
        // add to group
        let gid = match get_user_game(&conn, msg.uid)? {
            Some(id) => id,
            None => {
                return Err(WS::ValidationError("Not joined any game".to_owned()));
            }
        };

        // add to game
        match self.0.games.get_mut(&gid) {
            Some(mut game) => {
                game.insert(msg.addr);
            }
            None => {
                let mut new_game = HashSet::with_capacity(5);
                new_game.insert(msg.addr);
                self.0.games.insert(gid, new_game);
            }
        }

        // save to internal state map
        self.states.insert(gid, state);

        // compile data for message
        let username = get_username(&conn, &msg.uid)?;
        let data = DashMap::with_capacity(1);
        data.insert(
            "player".to_owned(),
            format!("{}|{}", msg.uid.to_string(), username),
        );

        // send message to everyone
        self.send_message(&gid, 7, data);

        // send id back
        Ok(())
    }
}

// Handler for Disconnect message.
impl Handler<Disconnect> for GameServer {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        // remove address
        let player_set = self.games.get_mut(&msg.gid);
        if player_set.is_some() {
            let mut players = player_set.unwrap();
            if players.remove(&msg.addr) {
                // check if game is empty and if empty remove and cleanup internal storage
                if players.is_empty() {
                    self.games.remove(&msg.gid);
                    let conn = self.pool.get()?;
                    remove_game(&conn, msg.gid)?;
                }

                // send notification
                let data = DashMap::new();
                data.insert("player".to_owned(), msg.uid.to_string());
                self.send_message(&msg.gid, 4, data);
            }
        }

        Ok(())
    }
}

// handler for fetching all or latest user moves
impl Handler<QueryMovesMessage> for GameServer {
    type Result = Result<ResizableGraphState, WebsocketError>;

    fn handle(&mut self, msg: QueryMovesMessage, _: &mut Context<Self>) -> Self::Result {
        return Ok(self.0.states.get(&msg.gid).unwrap().0.into());
    }
}

// handler for host starting a game
impl Handler<StartGameMessage> for GameServer {
    type Result = Result<(), WebsocketError>;

    fn handle(&mut self, msg: StartGameMessage, _: &mut Context<Self>) -> Self::Result {
        // check if user is authorized
        let host_id = self.0.games.get(&msg.gid).unwrap().value().host;

        if host_id != msg.uid {
            Err(WebsocketError::AuthorizationError())
        } else {
            // update state
            match self.0.states.get_mut(&msg.gid) {
                Some(mut mut_ref) => {
                    // update internal state
                    mut_ref.value_mut().1 = 1_u8;
                }
                None => {
                    return Err(WebsocketError::ValidationError(
                        "Game not found. Out of sync GameServer?".to_owned(),
                    ));
                }
            };

            self.send_message(&msg.gid, 5, DashMap::new());

            Ok(())
        }
    }
}

// handler for user move
impl Handler<MakeMoveMessage> for GameServer {
    type Result = Result<bool, WebsocketError>;

    fn handle(&mut self, msg: MakeMoveMessage, _: &mut Context<Self>) -> Self::Result {
        // get connections
        let dest = [msg.action.0[3], msg.action.0[4], msg.action.0[5]];

        // fetch starting point (db is trusted source)
        let db_friendly_figure: i16 = msg.action.1.into(); // SMALLINT requires i16
        let src = match fetch_latest_move(&conn, msg.gid, msg.uid, db_friendly_figure) {
            // take response and translate to array
            Ok((action, _)) => {
                let (last_src, last_dest) = action.split_at(2);
                // ensure move isn't directly repetitive - NOTE: This doesn'z cover
                if dest == last_dest {
                    return Err(WebsocketError::ValidationError(
                        "This move is repetitive".to_owned(),
                    ));
                } else {
                    last_src
                        .try_into()
                        .expect("Mov fetching returned corrupted results")
                }
            }
            // no move was made. Fall back
            Err(DBError::NotFound { .. }) => [db_friendly_figure, 0, 0],
            Err(_) => {
                log_error("[CRITICAL ERROR]", "Corrupted Database!!!!".to_owned());
                return Err(APIError::InternalError(
                    "Potential Corrupted database!".to_owned(),
                ));
            }
        };

        // validate move
        let state = self.states.get(&msg.gid).unwrap().value().0;
        let graph = GRAPH.clone();
        let result = graph.validate(&src, &dest, &state)?;

        match result.0 {
            true => {
                // add move to db
                make_new_move(
                    &conn,
                    msg.uid,
                    msg.gid,
                    (
                        [
                            *src.get(0).unwrap(),
                            *src.get(1).unwrap(),
                            *src.get(2).unwrap(),
                            dest[0],
                            dest[1],
                            dest[2],
                        ],
                        result.1,
                    ),
                )?;
                // send message of move to all other players

                Ok(true)
            }
            false => Err(APIError::ValidationError(
                "This move isn't possible".to_owned(),
            )),
        }
    }
}

// handler for game query message
impl Handler<QueryGameMessage> for GameServer {
    type Result = Result<
        (
            String,              // name
            String,              // description
            String,              // icon
            Vec<(Uuid, String)>, // users
            u8,                  // state
            Uuid,                // host
            Vec<i16>,            // pin
        ),
        APIError,
    >;

    fn handle(&mut self, msg: QueryGameMessage, _: &mut Context<Self>) -> Self::Result {
        let conn = self.pool.get()?;

        let (name, description, _, icon, pin) = get_slim_game(&conn, msg.gid)?;
        let users = get_game_users(&conn, msg.gid)?;
        // TODO: consider storing users (SlimUser + usize) within gameserver
        let host_id: Uuid = get_game_host(&conn, msg.gid)?;
        let game_state = self.states.get(&msg.gid).unwrap().1;

        Ok((
            name,
            description.unwrap_or("".to_owned()),
            icon,
            users,
            game_state,
            host_id,
            pin.unwrap_or(Vec::new()),
        ))
    }
}
