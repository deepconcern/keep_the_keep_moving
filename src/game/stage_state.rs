use bevy::prelude::*;

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, States)]
pub enum StageState {
    Shop,
    #[default]
    Wave,
}
