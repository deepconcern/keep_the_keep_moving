use bevy::{math::bounding::{BoundingCircle, IntersectsVolume}, prelude::*};

use crate::{asset_handles::AssetHandles, game::wave::enemy::{Enemy, EnemyState}, health::Health};

const DEFAULT_ARROW_DAMAGE: u32 = 2;
const DEFAULT_ARROW_SPEED: f32 = 200.0;

pub enum DefenderType {
    Archer,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Defender {
    pub action_timer: Timer,
    pub defender_type: DefenderType,
}

impl Default for Defender {
    fn default() -> Self {
        Self {
            action_timer: Timer::from_seconds(1.0, TimerMode::Once),
            defender_type: DefenderType::Archer,
        }
    }
}

fn defender_action(mut commands: Commands, enemy_query: Query<(&Enemy, Entity, &Transform)>, mut defender_query: Query<(&mut Defender, &Transform)>, time: Res<Time>) {
    for (mut defender, defender_transform) in defender_query.iter_mut() {
        defender.action_timer.tick(time.delta());

        if defender.action_timer.finished() {
            match defender.defender_type {
                DefenderType::Archer => {
                    let mut ds = None;
                    let mut target = None;

                    for (enemy, enemy_entity, enemy_transform) in enemy_query.iter() {
                        if enemy.enemy_state != EnemyState::Active {
                            continue;
                        }

                        let new_ds = enemy_transform.translation.distance_squared(defender_transform.translation);

                        let Some(ds_value) = ds else {
                            ds = Some(new_ds);
                            target = Some(enemy_entity);

                            continue;
                        };

                        if ds_value < new_ds {
                            ds = Some(new_ds);
                            target = Some(enemy_entity);
                        }
                    }

                    if target == None {
                        continue;
                    }

                    commands.spawn((
                        Transform {
                            translation: defender_transform.translation.clone(),
                            ..default()
                        },
                        Weapon {
                            damage: match defender.defender_type {
                                DefenderType::Archer => DEFAULT_ARROW_DAMAGE,
                            },
                            target: target,
                            ..default()
                        },
                    ));
                }
            }

            defender.action_timer.reset();
        }
    }
}

pub enum WeaponType {
    Arrow,
}

#[derive(Component)]
#[require(Sprite)]
pub struct Weapon {
    damage: u32,
    death_timer: Timer,
    direction: Vec2,
    speed: f32,
    target: Option<Entity>,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            damage: 0,
            death_timer: Timer::from_seconds(20.0, TimerMode::Once),
            direction: Vec2::ZERO,
            speed: DEFAULT_ARROW_SPEED,
            target: None,
        }
    }
}

fn initialize_weapon(asset_handles: Res<AssetHandles>, mut query: Query<&mut Sprite, Added<Weapon>>) {
    for mut sprite in query.iter_mut() {
        sprite.image = asset_handles.image_map.get("weapon").unwrap().clone();
        sprite.texture_atlas = Some(TextureAtlas {
            index: 0,
            layout: asset_handles
                .texture_atlas_layout_map
                .get("weapon")
                .unwrap()
                .clone(),
        });
    }
}

fn weapon_cancellation(mut commands: Commands, mut query: Query<(Entity, &mut Weapon)>, time: Res<Time>) {
    for (entity, mut weapon) in query.iter_mut() {
        weapon.death_timer.tick(time.delta());

        if weapon.death_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn weapon_hit(mut commands: Commands, mut enemy_query: Query<(&Enemy, &mut Health, &Transform)>, weapon_query: Query<(Entity, &Transform, &Weapon)>) {
    for (weapon_entity, weapon_transform, weapon) in weapon_query.iter() {
        let weapon_volume = BoundingCircle::new(weapon_transform.translation.xy(), 4.0);

        for (enemy, mut enemy_health, enemy_transform) in enemy_query.iter_mut() {
            if !enemy.volume(enemy_transform).intersects(&weapon_volume) {
                continue;
            }

            commands.entity(weapon_entity).despawn();

            enemy_health.current = enemy_health.current.saturating_sub(weapon.damage);
        }
    }
}

fn weapon_movement(mut commands: Commands, enemy_query: Query<&Transform, With<Enemy>>, time: Res<Time>, mut weapon_query: Query<(Entity, &mut Transform, &mut Weapon), Without<Enemy>>) {
    for (weapon_entity, mut weapon_transform, mut weapon) in weapon_query.iter_mut() {

        weapon.direction = match weapon.target {
            Some(enemy_entity) => {
                let Ok(enemy_transform) = enemy_query.get(enemy_entity) else {
                    commands.entity(weapon_entity).despawn();
                    continue;
                };

                (enemy_transform.translation - weapon_transform.translation).normalize().truncate()
            },
            None => weapon.direction
        };

        let translation = weapon.direction * weapon.speed * time.delta_secs();

        weapon_transform.translation += translation.extend(0.0);
        weapon_transform.rotation = Quat::from_rotation_arc(Vec3::Y, weapon.direction.extend(0.0));
    }
}

pub struct DefenderPlugin;

impl Plugin for DefenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (defender_action, initialize_weapon, weapon_cancellation, weapon_hit, weapon_movement));
    }
}