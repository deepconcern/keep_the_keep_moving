mod action;
mod app_state;
mod colors;
mod game;
mod menu;

use action::Action;
use app_state::AppState;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use game::GamePlugin;
use leafwing_input_manager::prelude::*;
use menu::MenuPlugin;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.4,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }).set(ImagePlugin::default_nearest()),
        EntropyPlugin::<WyRand>::default(),
        GamePlugin,
        InputManagerPlugin::<Action>::default(),
        MenuPlugin,
        TilemapPlugin,
    ));
    app.add_systems(Startup, setup);
    app.init_state::<AppState>();

    app.run();
}
