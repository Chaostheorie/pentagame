use super::graph::Figure;
use crate::ws::errors::WebsocketError;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use uuid::Uuid;

// types
// i16 is used to be translatable to PG SMALL INT
pub type MOVE = ([i16; 6], Figure);
pub type FIELD = [i16; 3];
pub type LOCATION = ([i16; 3], Figure);

// wrapper for MOVE to allow (de)serializing
#[derive(Deserialize, Serialize, Debug, PartialOrd, PartialEq)]
pub struct Move(MOVE);

#[derive(Deserialize, Serialize, Debug, PartialOrd, PartialEq)]
pub struct EMove {
    pub figure: String,
    pub player: Uuid,
    pub source: FIELD,
    pub steps: Vec<FIELD>,
}

impl Move {
    // This need to be adjusted for the new figure id system
    pub fn from_action(data: DashMap<String, String>) -> Result<Move, WebsocketError> {
        let mut action: MOVE = ([0_i16; 6], "Empty".to_owned());

        action.1 = match data.get("figure") {
            Some(raw_id) => match raw_id.parse::<u8>() {
                Ok(id) => id.to_string(),
                Err(_) => {
                    return Err(WebsocketError::ValidationError(
                        "Value for field figure doesn't fit into u8".to_owned(),
                    ));
                }
            },
            None => {
                return Err(WebsocketError::ValidationError(
                    "Missing field figure".to_owned(),
                ));
            }
        };

        action.0 = match data.get("move") {
            Some(raw_move) => match from_str::<[i16; 6]>(&raw_move) {
                Ok(parsed_move) => parsed_move,
                Err(_) => {
                    return Err(WebsocketError::ValidationError(
                        "Value for field move doesn't fit into [i16; 6]".to_owned(),
                    ));
                }
            },
            None => {
                return Err(WebsocketError::ValidationError(
                    "Missing field move".to_owned(),
                ));
            }
        };

        Ok(Move(action))
    }
}

impl From<EMove> for Move {
    fn from(source: EMove) -> Move {
        unimplemented!()
    }
}
