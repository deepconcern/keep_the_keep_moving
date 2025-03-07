use bevy::prelude::*;

use crate::app_state::AppState;

#[derive(Debug, Clone, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::Game)]
pub enum GameState {
    Shop,
    Wave,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Wave
    }
}
