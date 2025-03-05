use bevy::prelude::*;

const DEFAULT_ANIMATION_SPEED: f32 = 0.1;

#[derive(Component)]
#[require(Sprite)]
pub struct SimpleAnimation {
    pub animation_timer: Timer,
    pub current_frame_index: usize,
    pub frames: Vec<usize>,
}

impl Default for SimpleAnimation {
    fn default() -> Self {
        Self {
            animation_timer: Timer::from_seconds(DEFAULT_ANIMATION_SPEED, TimerMode::Repeating),
            current_frame_index: 0,
            frames: Vec::new(),
        }
    }
}

fn animate(mut query: Query<(&mut SimpleAnimation, &mut Sprite)>, time: Res<Time>) {
    for (mut simple_animation, mut sprite) in query.iter_mut() {
        let Some(texture_atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        simple_animation.animation_timer.tick(time.delta());

        if simple_animation.animation_timer.just_finished() {
            simple_animation.current_frame_index += 1;
            if simple_animation.current_frame_index >= simple_animation.frames.len() {
                simple_animation.current_frame_index = 0;
            }
            texture_atlas.index = simple_animation.frames[simple_animation.current_frame_index];
        }
    }
}

pub struct SimpleAnimationsPlugin;

impl Plugin for SimpleAnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate);
    }
}