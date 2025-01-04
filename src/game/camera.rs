use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::GameState;

use super::PlayerAction;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup)
            .add_systems(Update, control_camera.run_if(in_state(GameState::Game)));
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct FollowCam;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        FollowCam,
    ));

    // spawn 2D overlay
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..Default::default()
        },
    ));
}

fn control_camera(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<FollowCam>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta = time.delta_secs()
        * action_state
            .clamped_axis_pair(&PlayerAction::MoveCamera)
            .xy();
    player_transform.translation += Vec3::new(move_delta.x, 0.0, -move_delta.y);
}
