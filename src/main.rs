mod action;
mod asset_handles;
mod app_state;
mod colors;
mod game;
mod menu;
mod simple_animations;

use action::Action;
use app_state::AppState;
use asset_handles::AssetHandlesPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, render::camera::ScalingMode};
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use game::GamePlugin;
use leafwing_input_manager::prelude::*;
use menu::MenuPlugin;
use simple_animations::SimpleAnimationsPlugin;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.4,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1000.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn main() {
    let mut app = App::new();

    app.add_plugins((
        AssetHandlesPlugin,
        DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some (Window {
                resizable: true,
                title: "Keep the Keep Moving!".to_string(),
                ..default()
            }),
            ..default()
        }),
        EntropyPlugin::<WyRand>::default(),
        GamePlugin,
        InputManagerPlugin::<Action>::default(),
        MenuPlugin,
        SimpleAnimationsPlugin,
        TilemapPlugin,
    ));
    app.add_systems(Startup, setup);
    app.init_state::<AppState>();

    app.run();
}
