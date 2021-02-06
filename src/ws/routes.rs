use crate::auth::User;
use crate::frontend::routes::UserResponse;
use crate::state::AppState;
use crate::ws::{actor::GameServer, session::WsGameSession};
use actix::prelude::*;
use actix_web::{web::block, web::Data, web::HttpResponse, web::Payload, HttpRequest};
use actix_web_actors::ws;
use std::time::Instant;

pub async fn game_route(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<GameServer>>,
    state: Data<AppState>,
    id: Option<User>,
) -> Result<HttpResponse, APIError> {
    let user = guard_api_with_user(id, &state)?;

    /*
    this checks if the user already joined the game explicitly as the user might be only reconnecting
    */
    let sacrifice = user.id.clone();
    let result = block(move || get_user_game(&conn, sacrifice)).await?;

    let gid = match result {
        Some(id) => id,
        // check if game exists and if exists => join game
        None => {
            return Err(APIError::AuthorizationError(
                "You haven't joined this game. Consider visiting /game/view/{id} and checking out the game's data, if available.".to_owned()
            ));
        }
    };

    Ok(ws::start(
        WsGameSession {
            user,
            game: gid,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )?)
}

pub async fn get_game_leave_route(
    id: Option<SlimUser>,
    pool: Data<DbPool>,
    req: HttpRequest,
    state: Data<AppState>,
) -> UserResponse {
    let user = guard_with_user(&req, &state, id)?;

    // leave game
    let conn = pool.get()?;

    block(move || leave_game(&conn, user.id)).await?;

    Ok(redirect("/".to_owned()))
}
