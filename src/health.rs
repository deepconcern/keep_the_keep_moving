use bevy::prelude::*;

#[derive(Default)]
pub enum HealthState {
    #[default]
    Alive,
    Dead,
}

#[derive(Component, Default)]
pub struct Health {
    pub current: u32,
    pub max: u32,
    pub state: HealthState,
}

impl From<u32> for Health {
    fn from(value: u32) -> Self {
        Self {
            current: value,
            max: value,
            ..default()
        }
    }
}