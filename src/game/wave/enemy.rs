use bevy::{math::bounding::*, prelude::*};

use crate::{asset_handles::AssetHandles, game::game_sets::PausableSet};
use crate::health::Health;
use crate::simple_animations::SimpleAnimation;

use super::player::{PLAYER_SIZE, Player, PlayerState};
use super::wave_sets::WaveRunningSet;
use super::wave_state::WaveState;

const NORMAL_DAMAGE: u32 = 2;
const NORMAL_SIZE: f32 = 8.0;
const DEFAULT_SPEED: f32 = 120.0;
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
#[require(Health, SimpleAnimation(|| SimpleAnimation {
        current_frame_index: 0,
        frames: vec![3, 4],
        ..default()
}), Sprite, Transform, Visibility)]
pub struct Enemy {
    pub damage: u32,
    pub death_timer: Timer,
    pub direction: Vec2,
    pub enemy_type: EnemyType,
    pub enemy_state: EnemyState,
    pub spawn_timer: Timer,
    pub speed: f32,
}

impl Enemy {
    pub fn volume(&self, transform: &Transform) -> BoundingCircle {
        BoundingCircle::new(
            transform.translation.xy(),
            match self.enemy_type {
                EnemyType::Normal => NORMAL_SIZE,
            },
        )
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            damage: NORMAL_DAMAGE,
            death_timer: Timer::from_seconds(DEATH_RATE, TimerMode::Once),
            enemy_type: EnemyType::default(),
            enemy_state: EnemyState::default(),
            direction: Vec2::ZERO,
            spawn_timer: Timer::from_seconds(SPAWN_RATE, TimerMode::Once),
            speed: DEFAULT_SPEED,
        }
    }
}

fn destroy_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for enemy_entity in query.iter() {
        commands.entity(enemy_entity).despawn_recursive();
    }
}

fn enemy_behavior(
    mut commands: Commands,
    mut enemy_query: Query<(
        &mut Enemy,
        Entity,
        &mut SimpleAnimation,
        &mut Sprite,
        &Transform,
    )>,
    player_query: Query<(&Player, &Transform)>,
    time: Res<Time>,
) {
    let Ok((player, player_transform)) = player_query.get_single() else {
        return;
    };

    for (mut enemy, enemy_entity, mut enemy_animation, mut enemy_sprite, enemy_transform) in
        enemy_query.iter_mut()
    {
        match enemy.enemy_state {
            EnemyState::Spawning => {
                enemy.direction = Vec2::ZERO;
                enemy.spawn_timer.tick(time.delta());

                if (enemy.spawn_timer.just_finished()) {
                    enemy.enemy_state = EnemyState::Active;
                    enemy_animation.animation_timer.reset();
                    enemy_animation.current_frame_index = 0;
                    enemy_animation.frames = vec![0, 1];
                }
            }
            EnemyState::Active => {
                match enemy.enemy_type {
                    EnemyType::Normal => {
                        enemy.direction =
                            (player_transform.translation - enemy_transform.translation).truncate();

                        enemy.direction = enemy.direction.normalize();

                        enemy_sprite.flip_x = enemy.direction.x < 0.0;
                    }
                };
            }
            EnemyState::Dead => {
                enemy.direction = Vec2::ZERO;
                enemy.death_timer.tick(time.delta());

                if (enemy.death_timer.just_finished()) {
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}

fn enemy_death(mut query: Query<(&mut Enemy, &Health, &mut SimpleAnimation)>) {
    for (mut enemy, health, mut simple_animation) in query.iter_mut() {
        if health.current != 0 || enemy.enemy_state == EnemyState::Dead {
            continue;
        }

        
        enemy.enemy_state = EnemyState::Dead;
        simple_animation.animation_timer.reset();
        simple_animation.current_frame_index = 0;
        simple_animation.frames = vec![2];
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

fn initialize_enemy(
    asset_handles: Res<AssetHandles>,
    mut query: Query<(&mut Health, &mut Sprite), Added<Enemy>>,
) {
    for (mut health, mut sprite) in query.iter_mut() {
        health.max = 5;
        health.current = health.max;

        sprite.image = asset_handles.image_map.get("enemy").unwrap().clone();
        sprite.texture_atlas = Some(TextureAtlas {
            index: 3,
            layout: asset_handles
                .texture_atlas_layout_map
                .get("enemy")
                .unwrap()
                .clone(),
        });
    }
}

fn player_hit(
    enemy_query: Query<(&Enemy, &Transform)>,
    mut player_query: Query<(&mut Health, &mut Player, &Transform)>,
) {
    let Ok((mut health, mut player, player_transform)) = player_query.get_single_mut() else {
        return;
    };

    if player.player_state == PlayerState::Invincible {
        return;
    }

    let player_volume = BoundingCircle::new(player_transform.translation.xy(), PLAYER_SIZE);

    for (enemy, enemy_transform) in enemy_query.iter() {
        if !enemy.volume(enemy_transform).intersects(&player_volume) {
            continue;
        }

        health.current = health.current.saturating_sub(enemy.damage);
        player.player_state = PlayerState::Invincible;
        player.invincibility_timer.reset();
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(WaveState::Running), destroy_enemies);
        app.add_systems(
            Update,
            (
                enemy_behavior,
                enemy_death,
                enemy_movement,
                initialize_enemy,
                player_hit,
            ).in_set(PausableSet).in_set(WaveRunningSet),
        );
    }
}
