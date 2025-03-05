use bevy::{math::bounding::*, prelude::*};

#[derive(Component)]
pub enum BoundingVolume {
    Aabb(Aabb2d),
    Circle(BoundingCircle),
}

pub struct Intersection(bool);

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        // TODO
    }
}