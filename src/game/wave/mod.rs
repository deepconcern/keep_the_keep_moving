mod enemy;
mod player;
mod wave_state;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use enemy::{Enemy, EnemyPlugin};
use player::{Player, PlayerPlugin};
use rand::{seq::IteratorRandom, Rng};
use wave_state::WaveState;

use crate::asset_handles::AssetHandles;

use super::stage_state::StageState;
const AREA_SIZE: UVec2 = UVec2::new(128, 64);
const ARENA_SIZE: UVec2 = UVec2::new(64, 32);
const ENEMY_SPAWN_INTERVAL: f32 = 5.0;
const TILE_SIZE: f32 = 16.0;

pub struct WavePlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct Wave {
    enemy_spawn_timer: Timer,
}

impl Default for Wave {
    fn default() -> Self {
        Self {
            enemy_spawn_timer: Timer::from_seconds(ENEMY_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}

fn destroy_wave(mut commands: Commands, query: Query<Entity, With<Wave>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct Countdown {
    state: usize,
    timer: Timer,
}

fn destroy_countdown(mut commands: Commands, query: Query<Entity, With<Countdown>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn do_countdown(
    mut countdown_query: Query<(&Children, &mut Countdown)>,
    mut next_state: ResMut<NextState<WaveState>>,
    mut text_query: Query<&mut Text>,
    time: Res<Time>,
) {
    for (children, mut countdown) in countdown_query.iter_mut() {
        for &child in children.iter() {
            let Ok(mut text) = text_query.get_mut(child) else {
                continue;
            };

            text.0 = if countdown.state == 0 {
                "Go!".to_string()
            } else {
                countdown.state.to_string()
            };
        }

        countdown.timer.tick(time.delta());

        if countdown.timer.just_finished() {
            if countdown.state == 0 {
                next_state.set(WaveState::Running);
            } else {
                countdown.state -= 1;
                countdown.timer.reset();
            }
        }
    }
}

fn setup_countdown(asset_handles: Res<AssetHandles>, mut commands: Commands) {
    commands
        .spawn((
            BackgroundColor(Color::BLACK),
            Countdown {
                state: 3,
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
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
                Text::new("3"),
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
    mut commands: Commands,
    mut global_rng: GlobalEntropy<WyRand>,
) {
    // Build the arena
    let texture_handle = asset_server.load("sprites/terrain.png");

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(AREA_SIZE.into());

    let tilemap_id = TilemapId(tilemap_entity);

    let mut rng = global_rng.fork_rng();

    fill_tilemap(
        TileTextureIndex(10),
        AREA_SIZE.into(),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let arena_start = (AREA_SIZE - ARENA_SIZE) / 2;

    for x in arena_start.x..arena_start.x + ARENA_SIZE.x {
        for y in arena_start.y..arena_start.y + ARENA_SIZE.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(*[5, 6, 9].iter().choose(&mut rng).unwrap()),
                    tilemap_id: tilemap_id,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: TILE_SIZE, y: TILE_SIZE };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: AREA_SIZE.into(),
        spacing: TilemapSpacing { x: 1.0, y: 1.0 },
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&AREA_SIZE.into(), &grid_size, &map_type, -1.0),
        ..default()
    });

    // Setup rest of wave

    commands
        .spawn(Wave::default())
        .add_child(tilemap_entity)
        .with_children(|parent| {
            // UI

            parent
                .spawn((Node {
                    align_items: AlignItems::Stretch,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    height: Val::Vh(100.0),
                    width: Val::Vw(100.0),
                    ..default()
                },))
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
                                Text::new("Wave 1"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: asset_handles.font_map.get("default").unwrap().clone(),
                                    ..default()
                                },
                            ));
                        });
                });

            parent.spawn(Player::default());
        });
}

fn spawn_enemies(mut commands: Commands, mut global_rng: GlobalEntropy<WyRand>, mut query: Query<(Entity, &mut Wave)>, time: Res<Time>) {
    let Ok((wave_entity, mut wave)) = query.get_single_mut() else {
        return;
    };

    wave.enemy_spawn_timer.tick(time.delta());

    if wave.enemy_spawn_timer.just_finished() {
        let arena_real_boundary: Vec2 = (ARENA_SIZE - (ARENA_SIZE / 2)).as_vec2() * TILE_SIZE;
        let mut rng = global_rng.fork_rng();

        let x = rng.gen_range(0.0..arena_real_boundary.x);
        let y = rng.gen_range(0.0..arena_real_boundary.y);

        let enemy_entity = commands.spawn((
            Enemy::default(),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
        )).id();

        commands.entity(wave_entity).add_child(enemy_entity);
    }
}

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EnemyPlugin, PlayerPlugin));
        app.add_systems(OnEnter(StageState::Wave), setup_wave);
        app.add_systems(OnEnter(WaveState::Preparation), setup_countdown);
        app.add_systems(OnExit(WaveState::Preparation), destroy_countdown);
        app.add_systems(OnExit(StageState::Wave), destroy_wave);
        app.add_systems(
            Update,
            (do_countdown.run_if(in_state(WaveState::Preparation)), spawn_enemies.run_if(in_state(WaveState::Running))),
        );

        app.add_sub_state::<WaveState>();
    }
}
