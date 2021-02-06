// imports
use super::errors::UserError;
use super::{forms, templates};
use crate::auth::User;
use crate::state::AppState;
use actix_identity::Identity;
use actix_web::error::ErrorBadRequest;
use actix_web::{
    dev::HttpResponseBuilder, dev::Payload, http::header, http::StatusCode, web::block, web::Data,
    web::Form, web::Path, Error, FromRequest, HttpRequest, HttpResponse,
};
use askama_actix::TemplateIntoResponse;
use futures::future::{err, ok, Ready};
use rand::{seq::SliceRandom, thread_rng};
use serde::Serialize;
use serde_json::from_str;
use std::sync::Arc;
use uuid::Uuid;

// types
pub type UserResponse = Result<HttpResponse, UserError>;

// implementation of FromRequest Trait to allow for Automatic Parsing of session cookie
impl FromRequest for User {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, payload).into_inner() {
            if let Some(user_json) = identity.identity() {
                if let Ok(user) = from_str(&user_json) {
                    return ok(user);
                };
            }
        }
        return err(ErrorBadRequest("No user identifiable. Corrupted Cookie"));
    }
}

// empty string constant
const EMPTY: &'static str = "";

/*
General API Responses
    ActionStatus:
        code:
            0: Success
            1: Failure
            2: invalid
            3: unauthorized
        description: Description of err/ success

    QueryResult:
        code:
            Same as ActionStatus
        data:
            Post (id, title, body, published)
*/

#[derive(Serialize)]
pub struct ActionStatus {
    code: i8,
    description: String,
}

#[derive(Serialize)]
pub struct QueryResult {
    code: i8,
    data: (u32, String, String, bool),
}

/*
General:
/ -> get_index
/cookies -> get_cookie_information

Not registered -> get_error_404
*/

pub async fn get_index(id: Option<SlimUser>) -> UserResponse {
    UserError::wrap_template(templates::IndexTemplate { id, alert: EMPTY }.into_response())
}

pub async fn get_cookies(id: Option<SlimUser>) -> UserResponse {
    UserError::wrap_template(templates::CookiesTemplate { id }.into_response())
}

pub async fn get_error_404(id: Option<SlimUser>) -> UserResponse {
    UserError::wrap_template(
        templates::ErrorTemplate {
            code: 404,
            id,
            message: "The requested page is not available".to_owned(),
        }
        .into_response(),
    )
}

/*
INFO: All routes except overview require are guarded

/games:
    /: get_game_overview -> Overview of current games and your profile
    /create: get_create_game -> Simple form for creating a new game
    /view/{id}: get_view_game -> View of game and it's participants
    /join/{id}: f -> Make user join game and redirect to game 'playing' screen
    /leave: Leave a game (a player may only join one game at a time. Can be changed anytime but works as architectural rate limiting)
*/

pub async fn post_game_join(
    id: Option<User>,
    path: Path<(i32,)>,
    pool: Data<DbPool>,
    state: Data<AppState>,
    req: HttpRequest,
    form: Form<forms::GamePinForm>,
) -> UserResponse {
    // retrieve id and guard route
    let conn = pool.get()?;
    let uid = guard_with_user(&req, &state, id)?;

    let gid = path.0 .0;
    let sacrifice = gid.clone();
    let pin = match block(move || check_game(&conn, sacrifice)).await {
        Ok(pin) => pin,
        Err(_) => {
            return Err(UserError::NotFoundError());
        }
    };

    match pin {
        Some(pin) => {
            let mut stringified = String::new();
            pin.iter()
                .for_each(|number| stringified.push_str(&number.to_string()));

            if form.pin != stringified {
                return UserError::wrap_template(
                    templates::GamePinTemplate {
                        game: gid,
                        id: Some(uid),
                        pin_error: true,
                    }
                    .into_response(),
                );
            }
        }
        None => (),
    };

    let conn = pool.get()?;

    // check if user already joined game
    let sacrifice = uid.id.clone();
    match block(move || get_user_game(&conn, sacrifice)).await? {
        Some(current_game_id) => {
            if current_game_id != gid {
                let sacrifice = uid.id.clone();
                let conn = pool.get()?;
                block(move || leave_game(&conn, sacrifice)).await?;
                let conn = pool.get()?;
                block(move || join_game(&conn, sacrifice, gid)).await?;
            } else {
                // already joined -> check if host
                let conn = pool.get()?;
                let host = block(move || get_game_host(&conn, current_game_id)).await?;
                if host == uid.id {
                    return UserError::wrap_template(
                        templates::GameBoardTemplate {
                            id: Some(uid),
                            host: true,
                        }
                        .into_response(),
                    );
                }
            }
        }
        None => {
            let conn = pool.get()?;
            match block(move || join_game(&conn, sacrifice, gid)).await {
                Ok(_) => (),
                Err(_) => {
                    return Err(UserError::ValidationError("game id".to_owned()));
                }
            };
        }
    }

    UserError::wrap_template(
        templates::GameBoardTemplate {
            id: Some(uid),
            host: false,
        }
        .into_response(),
    )
}

pub async fn get_game_join(
    id: Option<User>,
    path: Path<(i32,)>,
    pool: Data<DbPool>,
    state: Data<AppState>,
    req: HttpRequest,
) -> UserResponse {
    // retrieve id and guard route
    let conn = pool.get()?;
    let gid = path.0 .0;
    let uid = guard_with_user(&req, &state, id);

    match uid {
        Ok(uid) => {
            let sacrifice = gid.clone();
            let pin = match block(move || check_game(&conn, sacrifice)).await {
                Ok(pin) => pin,
                Err(_) => {
                    return Err(UserError::NotFoundError());
                }
            };

            let conn = pool.get()?;

            // check if user already joined game
            let sacrifice = uid.id.clone();
            match block(move || get_user_game(&conn, sacrifice)).await? {
                Some(current_game_id) => {
                    if current_game_id != gid {
                        // when not current and game *and* has pin redirect to pin template
                        if !pin.is_none() {
                            return UserError::wrap_template(
                                templates::GamePinTemplate {
                                    game: gid,
                                    id: Some(uid),
                                    pin_error: false,
                                }
                                .into_response(),
                            );
                        } else {
                            // otherwise join directly
                            let sacrifice = uid.id.clone();
                            let conn = pool.get()?;
                            block(move || leave_game(&conn, sacrifice)).await?;
                            let conn = pool.get()?;
                            block(move || join_game(&conn, sacrifice, gid)).await?;
                        }
                    } else {
                        // already joined -> check if host
                        let conn = pool.get()?;
                        let host = block(move || get_game_host(&conn, current_game_id)).await?;
                        if host == uid.id {
                            return UserError::wrap_template(
                                templates::GameBoardTemplate {
                                    id: Some(uid),
                                    host: true,
                                }
                                .into_response(),
                            );
                        }
                    }
                }
                None => {
                    let conn = pool.get()?;
                    if !pin.is_none() {
                        return UserError::wrap_template(
                            templates::GamePinTemplate {
                                game: gid,
                                id: Some(uid),
                                pin_error: false,
                            }
                            .into_response(),
                        );
                    } else {
                        block(move || join_game(&conn, sacrifice, gid)).await?;
                    }
                }
            }

            UserError::wrap_template(
                templates::GameBoardTemplate {
                    id: Some(uid),
                    host: false,
                }
                .into_response(),
            )
        }
    }
}

pub async fn get_game_overview(id: Option<SlimUser>) -> UserResponse {
    // this contained the logic for fetching games earlier but to ensure XSS
    // is properly escaped it has been moved to an api route
    UserError::wrap_template(templates::GamesOverviewTemplate { id }.into_response())
}

pub async fn get_create_game(
    id: Option<User>,
    state: Data<AppState>,
    req: HttpRequest,
) -> UserResponse {
    println!("{:?}", id);
    guard_user(&req, &state, &id)?;

    UserError::wrap_template(
        templates::GamesCreateTemplate {
            id,
            name: EMPTY.to_owned(),
            description: EMPTY.to_owned(),
            name_error: false,
            description_error: false,
        }
        .into_response(),
    )
}

pub async fn post_create_game(
    data: Form<forms::GameForm>,
    id: Option<User>,
    state: Data<AppState>,
    req: HttpRequest,
) -> UserResponse {
    // constants for validation
    use crate::db::model::{DEFAULT_ICON, ICONS};

    // retrieve id and guard route
    let user = guard_with_user(&req, &state, id.clone())?;
    let conn = pool.get()?;

    // validates cookie checkbox
    let public = match &data.public {
        Some(content) => content == "on",
        None => true,
    };

    let pin = match &data.pin {
        Some(value) => {
            if value == "false" {
                // all possible values for pin
                let choices: [i16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

                // Thread rng
                let mut rng = thread_rng();

                // generate new pin
                Some(
                    (0..8)
                        .into_iter()
                        .map(|_| *choices.choose(&mut rng).unwrap())
                        .collect::<Vec<i16>>(),
                )
            } else {
                None
            }
        }
        None => None,
    };

    let icon = match &data.icon {
        Some(icon) => {
            if ICONS.contains(&icon.as_str()) {
                icon
            } else {
                DEFAULT_ICON
            }
        }
        None => DEFAULT_ICON,
    }
    .to_owned();

    // freeing thread because diesel doesn't support async net
    let gid = block(move || {
        create_game(
            &conn,
            data.name.clone(),
            data.description.clone(),
            public,
            icon,
            pin,
            &user,
        )
    })
    .await?;

    Ok(redirect(format!("/games/join/{}", gid)))
}

pub async fn get_view_game(
    path: Path<(i32,)>,
    id: Option<SlimUser>,
    pool: Data<DbPool>,
    state: Data<AppState>,
    req: HttpRequest,
) -> UserResponse {
    guard_user(&req, &state, &id)?;
    let conn = pool.get()?;
    let gid = path.into_inner().0;

    let gdata = block(move || get_game(&conn, gid)).await?;

    let is_host = false;

    UserError::wrap_template(
        templates::GamesViewTemplate {
            id,
            is_host,
            game: gdata.0,
            players: gdata.1,
        }
        .into_response(),
    )
}
