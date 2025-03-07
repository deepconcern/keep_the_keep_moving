mod game_controller;
mod game_sets;
mod game_state;
mod pause_menu;
mod pause_state;
mod shop;
mod wave;

use bevy::prelude::*;
use game_controller::GameController;
use game_sets::PausableSet;
use game_state::GameState;
use pause_state::PauseState;
use shop::ShopPlugin;
use wave::WavePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ShopPlugin, WavePlugin));

        app.add_sub_state::<GameState>();
        app.add_sub_state::<PauseState>();

        app.configure_sets(Update, PausableSet.run_if(in_state(PauseState::Running)));

        app.init_resource::<GameController>();
    }
}
