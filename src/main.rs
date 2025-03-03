mod action;
mod app_state;
mod colors;
mod game;
mod menu;

use action::Action;
use app_state::AppState;
use bevy::prelude::*;
use game::GamePlugin;
use leafwing_input_manager::prelude::*;
use menu::MenuPlugin;



fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn main() {
    let mut app = App::new();
    
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_linear()),
        GamePlugin,
        InputManagerPlugin::<Action>::default(),
        MenuPlugin
    ));
    app.add_systems(Startup, setup);
    app.init_state::<AppState>();

    app.run();
}