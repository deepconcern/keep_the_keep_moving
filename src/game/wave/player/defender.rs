use bevy::prelude::*;

use crate::{asset_handles::AssetHandles, game::wave::enemy::Enemy};

const DEFAULT_ARROW_SPEED: f32 = 120.0;

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

fn defender_action(mut commands: Commands, enemy_query: Query<&Transform, (With<Enemy>, Without<Defender>)>, mut defender_query: Query<(&mut Defender, &Transform)>, time: Res<Time>) {
    for (mut defender, defender_transform) in defender_query.iter_mut() {
        defender.action_timer.tick(time.delta());

        if defender.action_timer.finished() {
            match defender.defender_type {
                DefenderType::Archer => {
                    let mut ds = None;
                    let mut target = None;

                    for enemy_transform in enemy_query.iter() {
                        let new_ds = enemy_transform.translation.distance_squared(defender_transform.translation);

                        let Some(ds_value) = ds else {
                            ds = Some(new_ds);
                            target = Some(enemy_transform.translation);

                            continue;
                        };

                        if ds_value < new_ds {
                            ds = Some(new_ds);
                            target = Some(enemy_transform.translation);
                        }
                    }

                    let Some(target_value) = target else {
                        continue;
                    };

                    println!("FIRE: {:?}", target_value.truncate().normalize());

                    commands.spawn((
                        Transform {
                            translation: defender_transform.translation.clone(),
                            ..default()
                        },
                        Weapon {
                            direction: (target_value - defender_transform.translation).truncate().normalize(),
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
    death_timer: Timer,
    direction: Vec2,
    speed: f32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            death_timer: Timer::from_seconds(20.0, TimerMode::Once),
            direction: Vec2::ZERO,
            speed: DEFAULT_ARROW_SPEED,
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

fn weapon_movement(mut query: Query<(&mut Transform, &Weapon)>, time: Res<Time>) {
    for (mut transform, weapon) in query.iter_mut() {

        let translation = weapon.direction * weapon.speed * time.delta_secs();

        transform.translation += translation.extend(0.0);
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, weapon.direction.extend(0.0));
    }
}

pub struct DefenderPlugin;

impl Plugin for DefenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (defender_action, initialize_weapon, weapon_cancellation, weapon_movement));
    }
}