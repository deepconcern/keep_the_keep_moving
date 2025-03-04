use bevy::prelude::*;

use super::super::stage_state::StageState;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, SubStates)]
#[source(StageState = StageState::Wave)]
pub enum WaveState {
    Complete,
    GameOver,
    #[default]
    Preparation,
    Running,
}
