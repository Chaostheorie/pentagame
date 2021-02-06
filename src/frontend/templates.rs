use crate::auth::User;
use askama_actix::Template;
use uuid::Uuid;
use crate::ws::models::Game;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub id: Option<User>,
    pub alert: &'a str,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub message: String,
    pub code: u16,
    pub id: Option<User>,
}

#[derive(Template)]
#[template(path = "games/overview.html")]
pub struct GamesOverviewTemplate {
    pub id: Option<User>,
}

#[derive(Template)]
#[template(path = "games/create.html")]
pub struct GamesCreateTemplate {
    pub id: Option<User>,
    pub name: String,
    pub description: String,
    pub name_error: bool,
    pub description_error: bool,
}

#[derive(Template)]
#[template(path = "games/view.html")]
pub struct GamesViewTemplate {
    pub id: Option<User>,
    pub game: Game,
    pub is_host: bool,
    pub players: Vec<(Uuid, String)>,
}

#[derive(Template)]
#[template(path = "games/game.html")]
pub struct GameBoardTemplate {
    pub id: Option<User>,
    pub host: bool,
}

#[derive(Template)]
#[template(path = "games/pin.html")]
pub struct GamePinTemplate {
    pub id: Option<User>,
    pub pin_error: bool,
    pub game: i32,
}
