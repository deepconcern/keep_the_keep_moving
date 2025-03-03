use bevy::prelude::*;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, States)]
pub enum GameState {
    Paused,
    #[default]
    Running,
}
