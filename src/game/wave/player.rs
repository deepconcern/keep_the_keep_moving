use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    action::{default_input_map, Action},
    game::{game_state::GameState, stage_state::StageState},
};

const DEFAULT_DIRECTION: Vec2 = Vec2::Y;
const DEFAULT_SPEED: f32 = 1.0;
const TURN_RATE: f32 = 0.05;

pub struct PlayerPlugin;

#[derive(Component)]
#[require(ActionState<Action>, InputMap::<Action>(default_input_map),  Sprite, Transform, Visibility)]
pub struct Player {
    direction: Vec2,
    speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            direction: DEFAULT_DIRECTION,
            speed: DEFAULT_SPEED,
        }
    }
}

fn move_player(mut query: Query<(&Player, &mut Transform)>) {
    for (player, mut transform) in query.iter_mut() {
        let translation = player.direction * player.speed;

        transform.translation += translation.extend(0.0);
    }
}

fn steer_player(mut query: Query<(&ActionState<Action>, &mut Player)>) {
    for (action_state, mut player) in query.iter_mut() {
        let mut target_direction = Vec2::ZERO;

        if action_state.pressed(&Action::MoveUp) {
            target_direction.y += 1.0;
        }

        if action_state.pressed(&Action::MoveDown) {
            target_direction.y -= 1.0;
        }

        if action_state.pressed(&Action::MoveLeft) {
            target_direction.x -= 1.0;
        }

        if action_state.pressed(&Action::MoveRight) {
            target_direction.x += 1.0;
        }


        if target_direction != Vec2::ZERO {
            target_direction = target_direction.normalize();

            player.direction = player.direction.rotate_towards(target_direction, TURN_RATE);
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_player, steer_player).run_if(in_state(GameState::Running).and(in_state(StageState::Wave))),
        );
    }
}
