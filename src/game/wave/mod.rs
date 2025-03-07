mod enemy;
mod player;
mod wave_sets;
mod wave_state;

use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use enemy::{Enemy, EnemyPlugin};
use player::{Player, PlayerPlugin};
use rand::{Rng, seq::IteratorRandom};
use wave_sets::WaveRunningSet;
use wave_state::WaveState;

use crate::{app_state::AppState, asset_handles::AssetHandles, health::Health};

use super::{game_controller::GameController, game_state::GameState};

const AREA_SIZE: UVec2 = UVec2::new(128, 64);
const ARENA_SIZE: UVec2 = UVec2::new(48, 24);
const ARENA_BOUNDRY_OFFSET: u32 = 7;
const ENEMY_SPAWN_INTERVAL: f32 = 5.0;
const TILE_SIZE: f32 = 16.0;
const TRANSITION_RATE: f32 = 3.0;
const WAVE_RATE: f32 = 15.0;

pub struct WavePlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct WaveController {
    enemy_spawn_timer: Timer,
    finish_timer: Timer,
    game_over_timer: Timer,
    preparation_timer: Timer,
    wave_timer: Timer,
}

impl Default for WaveController {
    fn default() -> Self {
        Self {
            enemy_spawn_timer: Timer::from_seconds(ENEMY_SPAWN_INTERVAL, TimerMode::Repeating),
            finish_timer: Timer::from_seconds(TRANSITION_RATE, TimerMode::Once),
            game_over_timer: Timer::from_seconds(TRANSITION_RATE, TimerMode::Once),
            preparation_timer: Timer::from_seconds(1.0, TimerMode::Once),
            wave_timer: Timer::from_seconds(WAVE_RATE, TimerMode::Once),
        }
    }
}

#[derive(Component)]
struct FinishedMessage;

#[derive(Component)]
struct GameOverMessage;

#[derive(Component)]
struct HealthUi;
#[derive(Component)]
struct PreparationMessage(u32);

#[derive(Component)]
struct WaveTilemap;

#[derive(Component)]
struct WaveTimerUi;

#[derive(Component)]
struct WaveUi;

fn destroy_finished(mut commands: Commands, query: Query<Entity, With<FinishedMessage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn destroy_game_over(mut commands: Commands, query: Query<Entity, With<GameOverMessage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn destroy_preparation(mut commands: Commands, query: Query<Entity, With<PreparationMessage>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn destroy_wave(
    mut commands: Commands,
    wave_query: Query<Entity, With<WaveController>>,
    wave_ui_query: Query<Entity, With<WaveUi>>,
) {
    for entity in wave_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in wave_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn prepare(
    mut preparation_message_query: Query<(&Children, &PreparationMessage)>,
    mut text_query: Query<&mut Text>,
) {
    let Ok((children, preparation_message)) = preparation_message_query.get_single_mut() else {
        return;
    };

    for child in children.iter() {
        let Ok(mut text) = text_query.get_mut(*child) else {
            continue;
        };

        text.0 = if preparation_message.0 == 0 {
            "Go!".to_string()
        } else {
            preparation_message.0.to_string()
        };
    }
}

fn health_ui(
    player_query: Query<&Health, With<Player>>,
    mut text_query: Query<&mut Text, With<HealthUi>>,
) {
    let Ok(health) = player_query.get_single() else {
        return;
    };

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    text.0 = format!("HP: {}/{}", health.current, health.max);
}

fn setup_preparation(asset_handles: Res<AssetHandles>, mut commands: Commands) {
    commands
        .spawn((
            BackgroundColor(Color::BLACK),
            Node {
                align_items: AlignItems::Center,
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(16.0)),
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                ..default()
            },
            PreparationMessage(3),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("3"),
                TextColor(Color::WHITE),
                TextFont {
                    font: asset_handles.font_map.get("default").unwrap().clone(),
                    ..default()
                },
            ));
        });
}

fn setup_finished(asset_handles: Res<AssetHandles>, mut commands: Commands) {
    commands
        .spawn((
            BackgroundColor(Color::BLACK),
            FinishedMessage,
            Node {
                align_items: AlignItems::Center,
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(16.0)),
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Finished!"),
                TextColor(Color::WHITE),
                TextFont {
                    font: asset_handles.font_map.get("default").unwrap().clone(),
                    ..default()
                },
            ));
        });
}

fn setup_game_over(asset_handles: Res<AssetHandles>, mut commands: Commands) {
    commands
        .spawn((
            BackgroundColor(Color::BLACK),
            GameOverMessage,
            Node {
                align_items: AlignItems::Center,
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                left: Val::Percent(50.0),
                padding: UiRect::all(Val::Px(16.0)),
                position_type: PositionType::Absolute,
                top: Val::Percent(50.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over!"),
                TextColor(Color::WHITE),
                TextFont {
                    font: asset_handles.font_map.get("default").unwrap().clone(),
                    ..default()
                },
            ));
        });
}

fn setup_wave(
    asset_handles: Res<AssetHandles>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Transform, With<Camera>>,
    mut commands: Commands,
    game_controller: Res<GameController>,
    mut global_rng: GlobalEntropy<WyRand>,
) {
    // Reset camera position
    let Ok(mut camera_transform) = query.get_single_mut() else {
        return;
    };

    camera_transform.translation = Vec3::ZERO;

    // Wave controller
    commands.spawn(WaveController::default());

    // Build the arena
    let texture_handle = asset_server.load("sprites/terrain.png");

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(AREA_SIZE.into());

    let tilemap_id = TilemapId(tilemap_entity);

    let mut rng = global_rng.fork_rng();

    let arena_start = (AREA_SIZE - ARENA_SIZE) / 2;
    let arena_end = arena_start + ARENA_SIZE;

    let mut tile_texture_index = |x, y| {
        if x == arena_start.x {
            if y == arena_start.y {
                TileTextureIndex(12)
            } else if y == arena_end.y - 1 {
                TileTextureIndex(0)
            } else {
                TileTextureIndex(*[4, 8].iter().choose(&mut rng).unwrap())
            }
        } else if x == arena_end.x - 1 {
            if y == arena_start.y {
                TileTextureIndex(15)
            } else if y == arena_end.y - 1 {
                TileTextureIndex(3)
            } else {
                TileTextureIndex(*[7, 11].iter().choose(&mut rng).unwrap())
            }
        } else if y == arena_start.y {
            TileTextureIndex(*[13, 14].iter().choose(&mut rng).unwrap())
        } else if y == arena_end.y - 1 {
            TileTextureIndex(*[1, 2].iter().choose(&mut rng).unwrap())
        } else {
            TileTextureIndex(*[5, 6, 9].iter().choose(&mut rng).unwrap())
        }
    };

    for x in 0..AREA_SIZE.x {
        for y in 0..AREA_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: tile_texture_index(x, y),
                    tilemap_id: tilemap_id,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: AREA_SIZE.into(),
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&AREA_SIZE.into(), &grid_size, &map_type, -1.0),
            ..default()
        },
        WaveTilemap,
    ));

    // Setup rest of wave

    // UI

    commands
        .spawn((
            Node {
                align_items: AlignItems::Stretch,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                height: Val::Vh(100.0),
                width: Val::Vw(100.0),
                ..default()
            },
            WaveUi,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("Wave {}", game_controller.wave + 1)),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: asset_handles.font_map.get("default").unwrap().clone(),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Text::new(""),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: asset_handles.font_map.get("default").unwrap().clone(),
                            ..default()
                        },
                        WaveTimerUi,
                    ));
                });
            parent
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        HealthUi,
                        Text::new(""),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: asset_handles.font_map.get("default").unwrap().clone(),
                            ..default()
                        },
                    ));
                });
        });

    // Player

    commands.spawn(Player::default());
}

fn spawn_enemies(
    mut commands: Commands,
    mut global_rng: GlobalEntropy<WyRand>,
    mut query: Query<(Entity, &mut WaveController)>,
    time: Res<Time>,
) {
    let Ok((wave_entity, mut wave)) = query.get_single_mut() else {
        return;
    };

    wave.enemy_spawn_timer.tick(time.delta());

    if wave.enemy_spawn_timer.just_finished() {
        let arena_real_boundary: Vec2 = (ARENA_SIZE - (ARENA_SIZE / 2)).as_vec2() * TILE_SIZE;
        let mut rng = global_rng.fork_rng();

        let x = rng.gen_range(0.0..arena_real_boundary.x);
        let y = rng.gen_range(0.0..arena_real_boundary.y);

        let enemy_entity = commands
            .spawn((
                Enemy::default(),
                Transform::from_translation(Vec3::new(x, y, 0.0)),
            ))
            .id();

        commands.entity(wave_entity).add_child(enemy_entity);
    }
}

fn wave_timer_tick(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_stage_state: ResMut<NextState<GameState>>,
    mut next_wave_state: ResMut<NextState<WaveState>>,
    mut preparation_message_query: Query<&mut PreparationMessage>,
    time: Res<Time>,
    mut wave_controller_query: Query<&mut WaveController>,
    wave_state: Res<State<WaveState>>,
) {
    let Ok(mut wave_controller) = wave_controller_query.get_single_mut() else {
        return;
    };

    match wave_state.get() {
        WaveState::Complete => {
            wave_controller.finish_timer.tick(time.delta());

            if wave_controller.finish_timer.just_finished() {
                next_stage_state.set(GameState::Shop);
            }
        }
        WaveState::GameOver => {
            wave_controller.game_over_timer.tick(time.delta());

            if wave_controller.game_over_timer.just_finished() {
                next_app_state.set(AppState::Menu);
            }
        }
        WaveState::Preparation => {
            wave_controller.preparation_timer.tick(time.delta());

            if wave_controller.preparation_timer.just_finished() {
                let Ok(mut preparation_message) = preparation_message_query.get_single_mut() else {
                    return;
                };

                if preparation_message.0 > 0 {
                    preparation_message.0 -= 1;
                    wave_controller.preparation_timer.reset();
                } else {
                    next_wave_state.set(WaveState::Running);
                    wave_controller.wave_timer.reset();
                    wave_controller
                        .wave_timer
                        .set_duration(Duration::from_secs(WAVE_RATE as u64));
                }
            }
        }
        WaveState::Running => {
            wave_controller.wave_timer.tick(time.delta());

            if wave_controller.wave_timer.just_finished() {
                next_wave_state.set(WaveState::Complete);
                wave_controller.finish_timer.reset();
            }
        }
    }
}

fn wave_timer_ui(
    mut text_query: Query<&mut Text, With<WaveTimerUi>>,
    wave_controller_query: Query<&mut WaveController>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let Ok(wave_controller) = wave_controller_query.get_single() else {
        return;
    };

    text.0 = format!(
        "Time: {}/{}",
        wave_controller.wave_timer.remaining_secs() as u32,
        wave_controller.wave_timer.duration().as_secs()
    );
}

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EnemyPlugin, PlayerPlugin));
        app.add_sub_state::<WaveState>();
        app.add_systems(OnEnter(GameState::Wave), setup_wave);
        app.add_systems(OnEnter(WaveState::Complete), setup_finished);
        app.add_systems(OnExit(WaveState::Complete), destroy_finished);
        app.add_systems(OnEnter(WaveState::GameOver), setup_game_over);
        app.add_systems(OnExit(WaveState::GameOver), destroy_game_over);
        app.add_systems(OnEnter(WaveState::Preparation), setup_preparation);
        app.add_systems(OnExit(WaveState::Preparation), destroy_preparation);
        app.add_systems(OnExit(GameState::Wave), destroy_wave);
        app.add_systems(
            Update,
            (
                (health_ui, wave_timer_tick, wave_timer_ui),
                prepare.run_if(in_state(WaveState::Preparation)),
                (spawn_enemies).run_if(in_state(WaveState::Running)),
            )
                .run_if(in_state(GameState::Wave)),
        );

        app.configure_sets(Update, WaveRunningSet.run_if(in_state(WaveState::Running)));
    }
}
