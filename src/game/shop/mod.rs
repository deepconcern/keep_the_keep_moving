use bevy::prelude::*;

use super::{game_controller::GameController, game_state::GameState};

pub struct ShopPlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct Shop;

fn setup_shop(mut game_controller: ResMut<GameController>, mut next_state: ResMut<NextState<GameState>>) {
    game_controller.wave_level += 1;
    next_state.set(GameState::Wave);
}

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), setup_shop);
    }
}
