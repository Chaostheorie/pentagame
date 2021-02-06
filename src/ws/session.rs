use super::actor::GameServer;
use super::errors::{MESSAGE_FORMAT_ERROR, UNAUTHORIZED_ERROR, UNIMPLEMENTED_ERROR, WebsocketError};
use super::messages::{
    Connect, Disconnect, MakeMoveMessage, QueryGameMessage, QueryMovesMessage, ServerMessage,
    SessionMessage, StartGameMessage,
};
use crate::auth::User;
use crate::frontend::helper::log_error;
use crate::graph::models::Move;
use actix::prelude::*;
use actix_web_actors::ws;
use serde::Serialize;
use serde_json::to_string;
use std::time::{Duration, Instant};
use uuid::Uuid;

// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

// response specific structs
#[derive(Serialize)]
pub struct QueryGameResponse {
    name: String,
    description: String,
    icon: String,
    players: Vec<(Uuid, String)>,
    state: u8,
    host: Uuid,
    pin: Vec<i16>,
}

// Session specific struct
#[derive(Clone)]
pub struct WsGameSession {
    // Client must send ping at least once per 30 seconds (CLIENT_TIMEOUT),
    // otherwise server drop's connection.
    pub hb: Instant,
    // joined game
    pub game: i32,
    // Game server
    pub addr: Addr<GameServer>,
    // bound identity 
    pub user: User,
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self>;

    // Method is called on actor start.
    // register ws session with GameServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // start heartbeat process on session start.
        self.hb(ctx);

        // register address for server actor
        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
                uid: self.user.id,
            })
            .into_actor(self)
            .then(|message_result, _, ctx| {
                match message_result {
                    Ok(res) => {
                        match res {
                            Ok(_) => (),
                            Err(_) => {
                                ctx.text(UNAUTHORIZED_ERROR.clone());
                            }
                        };
                    }
                    // something is wrong with game server
                    Err(why) => {
                        log_error(
                            "[Session Error]",
                            format!("Gameserver closed rpc connection. Description: {}", why),
                        );

                        ctx.stop()
                    }
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, context: &mut Self::Context) -> Running {
        // notify game server
        self.addr.do_send(Disconnect {
            addr: context.address().recipient(),
            gid: self.game,
            uid: self.user.id,
        });

        Running::Stop
    }
}

// Handle messages from game server, we simply send it to peer websocket
impl Handler<SessionMessage> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) {
        ctx.text(to_string(&msg).expect("The GameServer sends corrupt messages"));
    }
}

// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsGameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(why) => {
                eprintln!("Error: {:?}", why);
                return ctx.stop();
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                match serde_json::from_str::<SessionMessage>(&text) {
                    Ok(action) => {
                        /*
                        TODO: move specific handling to dedicated functions in the future (those should be always inlined)
                        action & data
                        ---
                        | action | description         | data                | host only |
                        | ------ | ------------------- | ----------------    | --------- |
                        | 0      | fetch latest move   | {"all": boolean}    |           |
                        | 1      | get game meta       | {}                  |     X     |
                        | 2      | make move           | {"move": [MOVE]}    |     X     |
                        | 3      | Place Stopper       | {"move": String}    |     X     |
                        | 4      | leave game          | {}                  |     X     |
                        | 5      | start game          | {"message": String} |     ✓     |
                        | 6      | stop game           | {"message": String} |     ✓     |
                        */
                        match action.action {
                            // fetch latest move
                            0 => {
                                if action.data.contains_key("all") {
                                    self.addr
                                        .send(QueryMovesMessage {
                                            gid: self.game,
                                        })
                                        .into_actor(self)
                                        .then(|res, _, ctx| {
                                            let data = match res.expect("Gameserver crashed") {
                                                Ok(data) => data,
                                                Err(_) => {
                                                    ctx.text("Internal Error: Database Error or Gameserver Crashed".to_owned());
                                                    return fut::ready(()); // This doesn't return error as handled gracefully
                                                }
                                            };
                                            ctx.text(
                                                serde_json::to_string(&ServerMessage {
                                                    action: 0,
                                                    data,
                                                })
                                                .unwrap_or("Internal Error: Failed to parse message".to_owned()),
                                            );

                                            fut::ready(())
                                        })
                                        .wait(ctx);
                                } else {
                                    ctx.text(
                                        serde_json::to_string(&ServerMessage {
                                            action: u8::MAX,
                                            data: "Missing data key: 'all'".to_owned(),
                                        })
                                        .unwrap(),
                                    );
                                    return ();
                                }
                            }
                            1 => {
                                self.addr.send(QueryGameMessage { gid: self.game })
                            .into_actor(self)
                            // Result<(String, String, i32), APIError>
                            .then(|res, _, ctx| {
                                let _ = match res {
                                     Ok(result) => {
                                        let (game, players) = match result {
                                                Ok(meta) => {
                                                    meta
                                                },
                                                Err(_) => {
                                                    ctx.stop();
                                                    return fut::ready(());
                                                }
                                        };

                                        let message = ServerMessage {
                                            action: 1,
                                            data: QueryGameResponse {
                                                game, players
                                            }
                                        };

                                        let data = match serde_json::to_string(&message) {
                                                Ok(data) => data,
                                                Err(_) => {
                                                    ctx.stop();
                                                    return fut::ready(());
                                                }
                                        };

                                        ctx.text(data);

                                    }
                                    // something is wrong with game server
                                    Err(why) => {
                                        log_error("[Session Error]", format!("Gameserver closed rpc connection. Description: {}", why));
                                        
                                        ctx.stop()
                                    }
                                };
                                fut::ready(())

                            })
                            .wait(ctx);
                            }
                            2 => {
                                let parsed_move = match Move::from_action(action.data) {
                                    Ok(parsed_move) => parsed_move,
                                    Err(e) => {
                                        ctx.text(e.to_string());
                                        return;
                                    }
                                };

                                self.addr
                                    .send(MakeMoveMessage {
                                        action: parsed_move.action,
                                        gid: self.game,
                                        uid: self.user.id,
                                    })
                                    .into_actor(self)
                                    .then(|res, _, ctx| {
                                        let _ = match res {
                                            Ok(result) => {
                                                let state = match result {
                                                    Ok(state) => state,
                                                    Err(_) => {
                                                        ctx.stop();
                                                        return fut::ready(());
                                                    }
                                                };

                                                println!("{:?}", state);

                                                ctx.text(UNIMPLEMENTED_ERROR.clone());
                                            }
                                            // something is wrong with game server
                                            Err(_) => ctx.stop(),
                                        };
                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            }
                            5 => {
                                let sacrifice = self.clone();
                                self.addr
                                    .send(StartGameMessage {
                                        gid: self.game,
                                        uid: self.user.id,
                                    })
                                    .into_actor(self)
                                    .then(move |res, _, ctx| {
                                        match res {
                                            Ok(result) => {
                                                match result {
                                                    Ok(_) => (), // The gameserver handles sending messages to all participants
                                                    Err(APIError::AuthorizationError {
                                                        ..
                                                    }) => {
                                                        ctx.text(UNAUTHORIZED_ERROR.clone());
                                                    }
                                                    _ => {}
                                                };
                                            }
                                            // either gameserver down or unauthorized call
                                            Err(_) => {
                                                WsGameSession::stop(&sacrifice, ctx);
                                            }
                                        };
                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            }
                            _ => ctx.text(UNIMPLEMENTED_ERROR.clone()),
                        };
                    }
                    Err(_) => ctx.text(MESSAGE_FORMAT_ERROR.clone()),
                }
            }
            ws::Message::Binary(_) => ctx.text(UNIMPLEMENTED_ERROR.clone()),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                WsGameSession::stop(self, ctx);
            }
            ws::Message::Continuation(_) => {
                ctx.text(UNIMPLEMENTED_ERROR.clone());
                WsGameSession::stop(self, ctx);
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsGameSession {
    // helper method that sends ping to client every second.
    //
    // also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // clone game because &self wouldn't be guaranteed to satisfy 'static requirement of Context.run_interval
        let gid = self.game.clone();
        let uid = self.user.id.clone();

        ctx.run_interval(HEARTBEAT_INTERVAL, move |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // notify game server
                act.addr.do_send(Disconnect {
                    gid: gid,
                    uid: uid,
                    addr: ctx.address().recipient(),
                });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn stop(session: &WsGameSession, ctx: &mut ws::WebsocketContext<WsGameSession>) {
        session.addr.do_send(Disconnect {
            gid: session.game,
            uid: session.user.id,
            addr: ctx.address().recipient(),
        });
        ctx.stop();
    }
}
