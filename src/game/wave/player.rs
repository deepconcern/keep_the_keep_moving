use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    action::{Action, default_input_map},
    asset_handles::AssetHandles,
    game::{game_state::GameState, wave::wave_state::WaveState},
};

const DEFAULT_DIRECTION: Vec2 = Vec2::Y;
const DEFAULT_SPEED: f32 = 120.0;
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

fn follow_player(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    for mut camera_transform in camera_query.iter_mut() {
        for player_transform in player_query.iter() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}

fn initialize_player(asset_handles: Res<AssetHandles>, mut query: Query<&mut Sprite, Added<Player>>) {
    for mut sprite in query.iter_mut() {
        sprite.image = asset_handles.image_map.get("player").unwrap().clone();
        sprite.texture_atlas = Some(TextureAtlas {
            index: 0,
            layout: asset_handles
                .texture_atlas_layout_map
                .get("player")
                .unwrap()
                .clone(),
        });
    }
}

fn move_player(mut query: Query<(&Player, &mut Transform)>, time: Res<Time>) {
    for (player, mut transform) in query.iter_mut() {
        let translation = player.direction * player.speed * time.delta_secs();

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
            (follow_player, initialize_player, move_player, steer_player)
                .run_if(in_state(GameState::Running).and(in_state(WaveState::Running))),
        );
    }
}
