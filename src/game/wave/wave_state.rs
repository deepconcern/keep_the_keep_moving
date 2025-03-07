use bevy::prelude::*;

use super::super::game_state::GameState;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(GameState = GameState::Wave)]
pub enum WaveState {
    Complete,
    GameOver,
    #[default]
    Preparation,
    Running,
}
