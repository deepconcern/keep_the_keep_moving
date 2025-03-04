mod game_state;
mod pause_menu;
mod shop;
mod stage_state;
mod wave;

use bevy::prelude::*;
use game_state::GameState;
use shop::ShopPlugin;
use stage_state::StageState;
use wave::WavePlugin;

use crate::app_state::AppState;

#[derive(Component)]
struct Game;

pub struct GamePlugin;

fn destroy_game(mut commands: Commands, query: Query<Entity, With<Game>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game(mut commands: Commands) {
    commands.spawn((Game, Transform::default(), Visibility::default()));
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ShopPlugin, WavePlugin));
        app.add_systems(OnEnter(AppState::Game), setup_game);
        app.add_systems(OnExit(AppState::Game), destroy_game);

        app.add_sub_state::<GameState>();
        app.add_sub_state::<StageState>();
    }
}
