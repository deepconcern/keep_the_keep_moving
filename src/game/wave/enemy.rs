use bevy::prelude::*;

use crate::asset_handles::AssetHandles;
use crate::simple_animations::SimpleAnimation;

use super::player::Player;

const DEFAULT_SPEED: f32 = 100.0;
const DEATH_RATE: f32 = 1.0;
const SPAWN_RATE: f32 = 1.0;

#[derive(Default, Eq, PartialEq)]
pub enum EnemyState {
    Active,
    Dead,
    #[default]
    Spawning,
}

#[derive(Default, Eq, PartialEq)]
pub enum EnemyType {
    #[default]
    Normal,
}

#[derive(Component)]
#[require(SimpleAnimation(|| SimpleAnimation {
        current_frame_index: 0,
        frames: vec![3, 4],
        ..default()
}), Sprite, Transform, Visibility)]
pub struct Enemy {
    death_timer: Timer,
    direction: Vec2,
    enemy_type: EnemyType,
    enemy_state: EnemyState,
    spawn_timer: Timer,
    speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            death_timer: Timer::from_seconds(DEATH_RATE, TimerMode::Once),
            enemy_type: EnemyType::default(),
            enemy_state: EnemyState::default(),
            direction: Vec2::ZERO,
            spawn_timer: Timer::from_seconds(SPAWN_RATE, TimerMode::Once),
            speed: DEFAULT_SPEED,
        }
    }
}

fn enemy_behavior(mut commands: Commands, mut enemy_query: Query<(&mut Enemy, Entity, &mut SimpleAnimation, &Transform)>, player_query: Query<(&Player, &Transform)>, time: Res<Time>) {
    let Ok((player, player_transform)) = player_query.get_single() else {
        return;
    };

    for (mut enemy, enemy_entity, mut enemy_animation, enemy_transform) in enemy_query.iter_mut() {
        match enemy.enemy_state {
            EnemyState::Spawning => {
                enemy.direction = Vec2::ZERO;
                enemy.spawn_timer.tick(time.delta());

                if (enemy.spawn_timer.just_finished()) {
                    enemy.enemy_state = EnemyState::Active;
                }
            }
            EnemyState::Active => {
                match enemy.enemy_type {
                    EnemyType::Normal => {
                        enemy_animation.animation_timer.reset();
                        enemy_animation.current_frame_index = 0;
                        enemy_animation.frames = vec![0, 1];
                        enemy.direction = (player_transform.translation - enemy_transform.translation).truncate();
                        enemy.direction = enemy.direction.normalize();
                    },
                };
            }
            EnemyState::Dead => {
                enemy.direction = Vec2::ZERO;
                enemy.death_timer.tick(time.delta());

                if (enemy.death_timer.just_finished()) {
                    enemy_animation.current_frame_index = 0;
                    enemy_animation.frames = vec![2];
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}

fn enemy_movement(mut query: Query<(&Enemy, &mut Transform)>, time: Res<Time>) {
    for (enemy, mut transform) in query.iter_mut() {
        if enemy.enemy_state != EnemyState::Active {
            continue;
        }

        let translation = enemy.direction * enemy.speed * time.delta_secs();

        transform.translation += translation.extend(0.0);
    }
}

fn initialize_enemy(asset_handles: Res<AssetHandles>, mut query: Query<&mut Sprite, Added<Enemy>>) {
    for mut sprite in query.iter_mut() {
        sprite.image = asset_handles.image_map.get("enemy").unwrap().clone();
        sprite.texture_atlas = Some(TextureAtlas {
            index: 0,
            layout: asset_handles
                .texture_atlas_layout_map
                .get("enemy")
                .unwrap()
                .clone(),
        });
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (enemy_behavior, enemy_movement, initialize_enemy));
    }
}
