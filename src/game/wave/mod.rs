mod enemy;
mod player;
mod wave_controller;
mod wave_sets;
mod wave_state;

use bevy::{audio::*, math::bounding::*, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use enemy::{Enemy, EnemyPlugin};
use player::{Player, PlayerPlugin, PlayerState, PLAYER_SIZE};
use rand::{Rng, seq::IteratorRandom};
use wave_controller::{wave_timer_tick, WaveController};
use wave_sets::WaveRunningSet;
use wave_state::WaveState;

use crate::{asset_handles::AssetHandles, health::Health};

use super::{game_controller::GameController, game_state::GameState};

const AREA_SIZE: UVec2 = UVec2::new(128, 64);
const ARENA_SIZE: UVec2 = UVec2::new(48, 24);
const ARENA_BOUNDARY_OFFSET: u32 = 7;
const TILE_SIZE: f32 = 16.0;

#[derive(Component)]
struct FinishedMessage;

#[derive(Component)]
struct GameOverMessage;

#[derive(Component)]
struct HealthUi;
#[derive(Component)]
struct PreparationMessage;

#[derive(Component)]
struct WaveTilemap;

#[derive(Component)]
struct WaveTimerUi;

#[derive(Component)]
struct WaveUi;

#[derive(Resource)]
struct Arena {
    area: URect,
    playable_area: Aabb2d,
}

impl Default for Arena {
    fn default() -> Self {
        let total_area = URect::from_corners(UVec2::ZERO, AREA_SIZE);

        let area = URect::from_center_size(total_area.center(), ARENA_SIZE);

        Self {
            area,
            playable_area: Aabb2d::new(Vec2::ZERO, (area.half_size().as_vec2() * TILE_SIZE) - (PLAYER_SIZE * 1.4)),
        }
    }
}

fn boundary_collision(arena: Res<Arena>, mut query: Query<(&mut Player, &Transform)>) {
    let Ok((mut player, transform)) = query.get_single_mut() else {
        return;
    };

    if !arena.playable_area.intersects(&player.volume(transform)) {
        player.player_state = PlayerState::Dead;
    }
}

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
    audio_query: Query<Entity, With<AudioPlayer>>,
    mut commands: Commands,
    wave_ui_query: Query<Entity, With<WaveUi>>,
) {
    // Wave controller
    commands.remove_resource::<WaveController>();

    for entity in audio_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in wave_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn prepare(
    mut preparation_message_query: Query<&Children, With<PreparationMessage>>,
    mut text_query: Query<&mut Text>,
    wave_controller: Res<WaveController>,
) {
    let Ok(children) = preparation_message_query.get_single_mut() else {
        return;
    };

    for child in children.iter() {
        let Ok(mut text) = text_query.get_mut(*child) else {
            continue;
        };

        text.0 = if wave_controller.preparation_state == 0 {
            "Go!".to_string()
        } else {
            wave_controller.preparation_state.to_string()
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
            PreparationMessage,
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
    arena: Res<Arena>,
    asset_handles: Res<AssetHandles>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    game_controller: Res<GameController>,
    mut global_rng: GlobalEntropy<WyRand>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    commands.insert_resource(WaveController::from_level(game_controller.wave_level));

    // Start music
    #[cfg(not(target_family = "wasm"))]
    let bgm_handle = asset_server.load("sounds/bgm.wav");

    #[cfg(not(target_family = "wasm"))]
    commands.spawn((
        AudioPlayer::new(bgm_handle),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
    ));

    // Reset camera position
    let Ok(mut camera_transform) = query.get_single_mut() else {
        return;
    };

    camera_transform.translation = Vec3::ZERO;

    // Build the arena
    let texture_handle = asset_server.load("sprites/terrain.png");

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(AREA_SIZE.into());

    let tilemap_id = TilemapId(tilemap_entity);

    let mut rng = global_rng.fork_rng();

    let mut tile_texture_index = |x, y| {
        let point = UVec2::new(x, y);

        if !arena.area.contains(point) { return TileTextureIndex(10); }

        if x == arena.area.min.x {
            if y == arena.area.min.y {
                TileTextureIndex(12)
            } else if y == arena.area.max.y {
                TileTextureIndex(0)
            } else {
                TileTextureIndex(*[4, 8].iter().choose(&mut rng).unwrap())
            }
        } else if x == arena.area.max.x {
            if y == arena.area.min.y {
                TileTextureIndex(15)
            } else if y == arena.area.max.y {
                TileTextureIndex(3)
            } else {
                TileTextureIndex(*[7, 11].iter().choose(&mut rng).unwrap())
            }
        } else if y == arena.area.min.y {
            TileTextureIndex(*[13, 14].iter().choose(&mut rng).unwrap())
        } else if y == arena.area.max.y {
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
                        Text::new(format!("Wave {}", game_controller.wave_level + 1)),
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
    time: Res<Time>,
    mut wave_controller: ResMut<WaveController>,
) {

    wave_controller.enemy_spawn_timer.tick(time.delta());

    if wave_controller.enemy_spawn_timer.just_finished() {
        let arena_real_boundary: Vec2 = (ARENA_SIZE - (ARENA_SIZE / 2)).as_vec2() * TILE_SIZE;
        let mut rng = global_rng.fork_rng();

        for _ in 0..wave_controller.enemy_spawn_amount {

            let x = rng.gen_range(0.0..arena_real_boundary.x);
            let y = rng.gen_range(0.0..arena_real_boundary.y);

            commands
                .spawn((
                    Enemy::default(),
                    Transform::from_translation(Vec3::new(x, y, 0.0)),
                ));
        }
    }
}

fn wave_timer_ui(
    mut text_query: Query<&mut Text, With<WaveTimerUi>>,
    wave_controller: Res<WaveController>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    text.0 = format!(
        "Time: {}/{}",
        wave_controller.wave_timer.remaining_secs() as u32,
        wave_controller.wave_timer.duration().as_secs()
    );
}

pub struct WavePlugin;

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
                (boundary_collision, spawn_enemies).in_set(WaveRunningSet),
                (health_ui, wave_timer_tick, wave_timer_ui),
                prepare.run_if(in_state(WaveState::Preparation)),
            ).run_if(in_state(GameState::Wave)),
        );

        app.configure_sets(Update, WaveRunningSet.run_if(in_state(WaveState::Running)));

        app.init_resource::<Arena>();
    }
}
