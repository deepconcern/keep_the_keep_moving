use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Cancel,
    Confirm,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    OpenMenu,
}