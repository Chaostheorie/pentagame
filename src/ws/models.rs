use uuid::Uuid;

pub struct Game {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub pin: [u8; 6],
    pub host: Uuid,
    pub icon: String,
}
