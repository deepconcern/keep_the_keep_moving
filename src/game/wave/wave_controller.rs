use std::time::Duration;

use bevy::prelude::*;

use crate::{app_state::AppState, game::game_state::GameState};

use super::wave_state::WaveState;

const ENEMY_SPAWN_AMOUNT: u32 = 1;
const ENEMY_SPAWN_INTERVAL: f32 = 5.0;
const TRANSITION_RATE: f32 = 3.0;
const WAVE_RATE: f32 = 15.0;

#[derive(Resource)]

pub struct WaveController {
    pub enemy_spawn_amount: u32,
    pub enemy_spawn_timer: Timer,
    pub finish_timer: Timer,
    pub game_over_timer: Timer,
    pub preparation_state: u32,
    pub preparation_timer: Timer,
    pub wave_timer: Timer,
}

impl WaveController {
    pub fn from_level(level: u32) -> Self {
        let enemy_spawn_amount = ENEMY_SPAWN_AMOUNT + (level / 2);
        let enemy_spawn_interval = ENEMY_SPAWN_INTERVAL - (level as f32 * 0.5);

        Self {
            enemy_spawn_amount,
            enemy_spawn_timer: Timer::from_seconds(enemy_spawn_interval, TimerMode::Repeating),
            finish_timer: Timer::from_seconds(TRANSITION_RATE, TimerMode::Once),
            game_over_timer: Timer::from_seconds(TRANSITION_RATE, TimerMode::Once),
            preparation_state: 3,
            preparation_timer: Timer::from_seconds(1.0, TimerMode::Once),
            wave_timer: Timer::from_seconds(WAVE_RATE, TimerMode::Once),
        }
    }
}


pub fn wave_timer_tick(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_stage_state: ResMut<NextState<GameState>>,
    mut next_wave_state: ResMut<NextState<WaveState>>,
    time: Res<Time>,
    mut wave_controller: ResMut<WaveController>,
    wave_state: Res<State<WaveState>>,
) {

    match wave_state.get() {
        WaveState::Complete => {
            wave_controller.finish_timer.tick(time.delta());

            if wave_controller.finish_timer.just_finished() {
                next_stage_state.set(GameState::Shop);
            }
        }
        WaveState::GameOver => {
            wave_controller.game_over_timer.tick(time.delta());

            if wave_controller.game_over_timer.just_finished() {
                next_app_state.set(AppState::Menu);
            }
        }
        WaveState::Preparation => {
            wave_controller.preparation_timer.tick(time.delta());

            if wave_controller.preparation_timer.just_finished() {
                if wave_controller.preparation_state > 0 {
                    wave_controller.preparation_state -= 1;
                    wave_controller.preparation_timer.reset();
                } else {
                    next_wave_state.set(WaveState::Running);
                    wave_controller.wave_timer.reset();
                    wave_controller
                        .wave_timer
                        .set_duration(Duration::from_secs(WAVE_RATE as u64));
                }
            }
        }
        WaveState::Running => {
            wave_controller.wave_timer.tick(time.delta());

            if wave_controller.wave_timer.just_finished() {
                next_wave_state.set(WaveState::Complete);
                wave_controller.finish_timer.reset();
            }
        }
    }
}