mod player;

use bevy::prelude::*;
use player::{Player, PlayerPlugin};

use super::stage_state::StageState;

pub struct WavePlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct Wave;

fn destroy_wave(mut commands: Commands, query: Query<Entity, With<Wave>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_wave(
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

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerPlugin);
        app.add_systems(OnEnter(StageState::Wave), setup_wave);
        app.add_systems(OnExit(StageState::Wave), destroy_wave);
    }
}
