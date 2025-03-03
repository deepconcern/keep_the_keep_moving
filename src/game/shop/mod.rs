use bevy::prelude::*;

use super::stage_state::StageState;

pub struct ShopPlugin;

#[derive(Component)]
#[require(Transform, Visibility)]
struct Shop;

fn destroy_shop(mut commands: Commands, query: Query<Entity, With<Shop>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_shop(mut commands: Commands) {
    commands.spawn(Shop);
}

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(StageState::Shop), setup_shop);
        app.add_systems(OnExit(StageState::Shop), destroy_shop);
    }
}