use bevy::prelude::*;

use crate::app_state::AppState;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, SubStates)]
#[source(AppState = AppState::Game)]
pub enum PauseState {
    Paused,
    #[default]
    Running,
}
