use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::GameState;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_plugins(InfiniteGridPlugin)
        .add_systems(
            OnEnter(GameState::Game),
            debug_system.run_if(input_toggle_active(true, KeyCode::F1)),
        );
    }
}

fn debug_system(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle::default());
}
