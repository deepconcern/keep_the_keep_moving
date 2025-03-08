use bevy::{prelude::*, utils::HashMap};

use crate::app_state::AppState;

const FONTS: [(&'static str, &'static str); 1] = [
    ("default", "fonts/PressStart2P-Regular.ttf"),
];

const TEXTURE_ATLASES: [(&'static str, &'static str, UVec2, u32, u32, Option<UVec2>, Option<UVec2>); 3] = [
    ("enemy", "sprites/enemies.png", UVec2::splat(16), 3, 2, Some(UVec2::splat(1)), None),
    ("player", "sprites/keep-v2.png", UVec2::splat(32), 2, 2, Some(UVec2::splat(1)), None),
    ("weapon", "sprites/weapons.png", UVec2::splat(8), 2, 2, Some(UVec2::splat(1)), None),
];

#[derive(Debug, Default, Resource)]
pub struct AssetHandles {
    pub font_map: HashMap<String, Handle<Font>>,
    pub image_map: HashMap<String, Handle<Image>>,
    pub texture_atlas_layout_map: HashMap<String, Handle<TextureAtlasLayout>>,
}

fn initialize_asset_handles(
    mut asset_handles: ResMut<AssetHandles>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (key, path) in FONTS {
        let font_handle = asset_server.load(path);
    
        asset_handles
            .font_map
            .insert(key.to_string(), font_handle);
    }

    for (key, path, tile_size, columns, rows, padding, offset) in TEXTURE_ATLASES {
        let texture_handle = asset_server.load(path);
    
        asset_handles
            .image_map
            .insert(key.to_string(), texture_handle);
    
        let texture_atlas_layout = TextureAtlasLayout::from_grid(tile_size, columns, rows, padding, offset);
        let texture_atlas_layout_handle = texture_atlas_layouts.add(texture_atlas_layout);
    
        asset_handles
            .texture_atlas_layout_map
            .insert(key.to_string(), texture_atlas_layout_handle);
    }

    next_state.set(AppState::Menu);
}

pub struct AssetHandlesPlugin;

impl Plugin for AssetHandlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), initialize_asset_handles);
        app.init_resource::<AssetHandles>();
    }
}
