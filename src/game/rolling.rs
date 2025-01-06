use bevy::prelude::*;

use super::{DiePool, GamePlayState};

pub struct RollingPlugin;

impl Plugin for RollingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GamePlayState::Rolling),
            roll_dice.run_if(in_state(GamePlayState::Rolling)),
        );
    }
}

fn roll_dice(mut die_pool: ResMut<DiePool>, mut next_state: ResMut<NextState<GamePlayState>>) {
    die_pool.roll();
    next_state.set(GamePlayState::Placement);
}
