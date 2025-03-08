mod defender;

use bevy::{math::bounding::*, prelude::*};
use defender::{Defender, DefenderPlugin};
use leafwing_input_manager::prelude::*;

use crate::{
    action::{default_input_map, Action},
    asset_handles::AssetHandles,
    game::{game_sets::PausableSet, wave::wave_state::WaveState},
    health::Health, simple_animations::SimpleAnimation,
};

use super::wave_sets::WaveRunningSet;

const DEATH_RATE: f32 = 1.0;
const DEFAULT_DIRECTION: Vec2 = Vec2::Y;
const DEFAULT_SPEED: f32 = 120.0;
const INVINCIBILITY_RATE: f32 = 0.25;
pub const PLAYER_SIZE: f32 = 16.0;
const TURN_RATE: f32 = 0.03;

pub struct PlayerPlugin;

#[derive(Eq, PartialEq)]
pub enum PlayerState {
    Dead,
    Invincible,
    Normal,
}

#[derive(Component)]
#[require(ActionState<Action>, Health(|| 10), InputMap::<Action>(default_input_map), SimpleAnimation,  Sprite, Transform, Visibility)]
pub struct Player {
    pub death_timer: Timer,
    pub direction: Vec2,
    pub invincibility_timer: Timer,
    pub player_state: PlayerState,
    pub speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            death_timer: Timer::from_seconds(DEATH_RATE, TimerMode::Once),
            direction: DEFAULT_DIRECTION,
            invincibility_timer: Timer::from_seconds(INVINCIBILITY_RATE, TimerMode::Once),
            player_state: PlayerState::Normal,
            speed: DEFAULT_SPEED,
        }
    }
}

impl Player {
    pub fn volume(&self, transform: &Transform) -> BoundingCircle {
        BoundingCircle::new(transform.translation.xy(), PLAYER_SIZE)
    }
}

fn destroy_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    let Ok(player_entity) = query.get_single() else {
        return;
    };

    commands.entity(player_entity).despawn_recursive();
}

fn follow_player(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Defender>, Without<Player>)>,
    mut defender_query: Query<&mut Transform, (With<Defender>, Without<Camera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>, Without<Defender>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    for mut camera_transform in camera_query.iter_mut() {
        camera_transform.translation.x = player_transform.translation.x;
        camera_transform.translation.y = player_transform.translation.y;
    }

    for mut defender_transform in defender_query.iter_mut() {
        defender_transform.translation.x = player_transform.translation.x;
        defender_transform.translation.y = player_transform.translation.y;
    }
}

fn initialize_player(
    asset_handles: Res<AssetHandles>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SimpleAnimation, &mut Sprite), Added<Player>>,
) {
    let Ok((player_entity, mut player_animation, mut player_sprite)) = query.get_single_mut() else {
        return;
    };

    // Set the player's sprite
    player_animation.frames = vec![0, 1, 0, 2];
    player_sprite.image = asset_handles.image_map.get("player").unwrap().clone();
    player_sprite.texture_atlas = Some(TextureAtlas {
        index: 0,
        layout: asset_handles
            .texture_atlas_layout_map
            .get("player")
            .unwrap()
            .clone(),
    });

    // Start the player with an archers
    let archer_entity = commands.spawn(Defender::default()).id();

    commands.entity(player_entity).add_child(archer_entity);
}

fn move_player(mut query: Query<(&Player, &mut Transform)>, time: Res<Time>) {
    let Ok((player, mut transform)) = query.get_single_mut() else {
        return;
    };

    if player.player_state == PlayerState::Dead {
        return;
    }

    let translation = player.direction * player.speed * time.delta_secs();

    transform.translation += translation.extend(0.0);
}

fn player_death(
    mut next_state: ResMut<NextState<WaveState>>,
    mut query: Query<(&mut Player, &mut SimpleAnimation)>,
    time: Res<Time>,
) {
    let Ok((mut player, mut player_animation)) = query.get_single_mut() else {
        return;
    };

    if player.player_state != PlayerState::Dead {
        return;
    }

    player_animation.current_frame_index = 0;
    player_animation.frames = vec![3];

    if player.death_timer.finished() {
        return;
    }

    player.death_timer.tick(time.delta());

    if player.death_timer.just_finished() {
        next_state.set(WaveState::GameOver);
    }
}

fn player_health(mut query: Query<(&Health, &mut Player)>) {
    let Ok((health, mut player)) = query.get_single_mut() else {
        return;
    };

    if player.player_state == PlayerState::Dead {
        return;
    }

    if health.current > 0 {
        return;
    }

    player.death_timer.reset();
    player.player_state = PlayerState::Dead;
}

fn player_invincibility(mut query: Query<&mut Player>, time: Res<Time>) {
    let Ok(mut player) = query.get_single_mut() else {
        return;
    };

    if player.player_state != PlayerState::Invincible {
        return;
    }

    player.invincibility_timer.tick(time.delta());

    if player.invincibility_timer.just_finished() {
        player.player_state = PlayerState::Normal;
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
        app.add_plugins(DefenderPlugin);
        app.add_systems(OnExit(WaveState::Running), destroy_player);
        app.add_systems(
            Update,
            (
                follow_player,
                initialize_player,
                move_player,
                player_death,
                player_health,
                player_invincibility,
                steer_player,
            )
                .in_set(PausableSet)
                .in_set(WaveRunningSet),
        );
    }
}
