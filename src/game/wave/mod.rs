mod player;
mod wave_state;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use player::{Player, PlayerPlugin};
use rand::seq::IteratorRandom;
use wave_state::WaveState;

use super::stage_state::StageState;
const AREA_SIZE: UVec2 = UVec2::new(128, 64);
const ARENA_SIZE: UVec2 = UVec2::new(64, 32);

pub struct WavePlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct Wave;

fn destroy_wave(mut commands: Commands, query: Query<Entity, With<Wave>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_player(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("sprites/keep.png");

    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 1, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layouts.add(texture_atlas_layout);

    commands.spawn(Wave).with_children(|parent| {
        parent.spawn((
            Player::default(),
            Sprite::from_atlas_image(
                texture_handle,
                TextureAtlas {
                    index: 0,
                    layout: texture_atlas_layout_handle,
                },
            ),
        ));
    });
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

fn setup_countdown(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font_handle = asset_server.load("fonts/PressStart2P-Regular.ttf");

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
                    font: font_handle.clone(),
                    ..default()
                },
            ));
        });
}

fn setup_ui(asset_server: Res<AssetServer>, mut commands: Commands) {
    let font_handle = asset_server.load("fonts/PressStart2P-Regular.ttf");

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
            Wave,
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
                        Text::new("Wave 1"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: font_handle.clone(),
                            ..default()
                        },
                    ));
                });
        });
}

fn setup_terrain(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut global_rng: GlobalEntropy<WyRand>
) {
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
    

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
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

    commands.spawn(Wave).add_child(tilemap_entity);
}

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin);
        app.add_systems(
            OnEnter(StageState::Wave),
            (setup_player, setup_terrain, setup_ui),
        );
        app.add_systems(OnEnter(WaveState::Preparation), setup_countdown);
        app.add_systems(OnExit(WaveState::Preparation), destroy_countdown);
        app.add_systems(OnExit(StageState::Wave), destroy_wave);
        app.add_systems(Update, do_countdown.run_if(in_state(WaveState::Preparation)));

        app.add_sub_state::<WaveState>();
    }
}
