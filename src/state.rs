use crate::auth::AuthCache;
use crate::graph::graph::GraphState;
use crate::ws::messages::SessionMessage;
use crate::ws::models::Game;
use actix::prelude::*;
use dashmap::{DashMap, DashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_cache: Arc<AuthCache>, // This only possible due to DashMaps nature
}

#[derive(Clone)]
pub struct GameServerState {
    pub players: Arc<AuthCache>,
    pub states: Arc<DashMap<i32, (GraphState, u8)>>,
    pub games: Arc<DashMap<i32, Game>>,
    pub sessions: Arc<DashMap<i32, DashSet<Recipient<SessionMessage>>>>,
}

impl AppState {
    pub fn build() -> AppState {
        return AppState {
            auth_cache: Arc::new(AuthCache::build()),
        };
    }
}

impl GameServerState {
    pub fn build(players: Arc<AuthCache>) -> GameServerState {
        GameServerState {
            players,
            states: Arc::new(DashMap::new()),
            games: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        }
    }
}
