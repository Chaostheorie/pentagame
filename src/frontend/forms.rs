use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameForm {
    pub name: String,
    pub public: Option<String>,
    pub description: Option<String>,
    pub icon: String,
    pub pin: Option<String>,
}

#[derive(Deserialize)]
pub struct GamePinForm {
    pub pin: String,
}
