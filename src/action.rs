use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Cancel,
    CloseMenu,
    Confirm,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUp,
    OpenMenu,
}

pub fn default_input_map() -> InputMap<Action> {
    InputMap::new([
        (Action::Cancel, KeyCode::Escape),
        (Action::CloseMenu, KeyCode::Escape),
        (Action::Confirm, KeyCode::Enter),
        (Action::Confirm, KeyCode::Space),
        (Action::MoveDown, KeyCode::ArrowDown),
        (Action::MoveDown, KeyCode::KeyS),
        (Action::MoveLeft, KeyCode::ArrowLeft),
        (Action::MoveLeft, KeyCode::KeyA),
        (Action::MoveRight, KeyCode::ArrowRight),
        (Action::MoveRight, KeyCode::KeyD),
        (Action::MoveUp, KeyCode::ArrowUp),
        (Action::MoveUp, KeyCode::KeyW),
        (Action::OpenMenu, KeyCode::Escape),
    ])
}
