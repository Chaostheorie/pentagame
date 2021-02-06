// imports
use super::errors::WebsocketError;
use super::models::Game;
use crate::graph::graph::ResizableGraphState;
use crate::graph::models::MOVE;
use actix::prelude::*;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// message and relative deserializer for client and server -> session messages
#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SessionMessage {
    /*
    | action | description                  | data                 |
    | ------ | ---------------------------- | -------------------- |
    | 0      | {user} joined room           | {"user": {user}}     |
    | 1      | {user} made move             | {                    |
    |        |                              |  "user": {user},     |
    |        |                              |  "move": String      |
    |        |                              | }                    |
    | 2      | {user} needs to place figure | {"user": {user}}     |
    | 3      | {user} placed figure         | {                    |
    |        |                              |  "user": {user},     |
    |        |                              |  "move": String      |
    |        |                              | }                    |
    | 4      | {user} disconnected          | {"user": {user}}     |
    | 5      | Login                        | {                    |
    |        |                              |  "name": String,     |
    |        |                              |  "password": String, |
    |        |                              | }                    |

    Login is bound to websocket as cookie so no logout action required
    */
    pub action: u8,
    pub data: DashMap<String, String>,
}

// Messages for session -> game server communications
#[derive(Message)]
#[rtype(result = "Result<(Game, Vec<(Uuid, String)>), WebsocketError>")]
pub struct QueryGameMessage {
    pub game: Game,
    pub players: Vec<(Uuid, String)>,
}

#[derive(Message)]
#[rtype(result = "Result<ResizableGraphState, WebsocketError>")]
pub struct QueryMovesMessage {
    pub gid: i32,
}

#[derive(Message)]
#[rtype(result = "Result<(), WebsocketError>")]
pub struct StartGameMessage {
    pub gid: i32,
    pub uid: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<bool, WebsocketError>")]
pub struct MakeMoveMessage {
    // user id from game session
    pub uid: Uuid,
    // move to make/ validate
    pub action: MOVE,
    // related game id
    pub gid: i32,
}

// New game session is created
#[derive(Message)]
#[rtype(result = "Result<(), WebsocketError>")]
pub struct Connect {
    // session id (== user id)
    pub addr: Recipient<SessionMessage>,
    pub uid: Uuid,
}

// Session is disconnected
#[derive(Message)]
#[rtype(result = "Result<(), WebsocketError>")]
pub struct Disconnect {
    pub gid: i32,
    pub uid: Uuid,
    pub addr: Recipient<SessionMessage>,
}

// Messages (serialized data) from session/ server -> client
#[derive(Serialize, Clone, Debug)]
pub struct ServerMessage<G> {
    pub action: u8,
    pub data: G,
}

// Messages (non generic - serialized data) from session -> client
#[derive(Serialize, Clone, Debug)]
pub struct SimpleServerMessage<'a> {
    pub action: u8,
    pub data: DashMap<&'a str, String>,
}
