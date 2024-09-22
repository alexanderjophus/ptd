mod camera;
mod placement;
mod wave;

use std::f32::consts::PI;

use super::GameState;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use camera::CameraPlugin;
use leafwing_input_manager::prelude::*;
use placement::PlacementPlugin;
use wave::WavePlugin;

// Enum that will be used as a state for the gameplay loop
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GamePlayState {
    #[default]
    Placement,
    Wave,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    MoveCamera,
    MovePlaceholderTower,
    ToggleTowerType,
    PlaceTower,
    EndPlacement,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::MoveCamera => InputControlKind::DualAxis,
            PlayerAction::MovePlaceholderTower => InputControlKind::DualAxis,
            PlayerAction::ToggleTowerType => InputControlKind::Button,
            PlayerAction::PlaceTower => InputControlKind::Button,
            PlayerAction::EndPlacement => InputControlKind::Button,
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "models/sv-charmander.glb")]
    pub charmander: Handle<Gltf>,
    #[asset(path = "models/sv-diglett.glb")]
    pub diglett: Handle<Gltf>,
    #[asset(path = "models/sv-gastly.glb")]
    pub gastly: Handle<Gltf>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePlayState>()
            .add_plugins((
                InputManagerPlugin::<PlayerAction>::default(),
                CameraPlugin,
                PlacementPlugin,
                WavePlugin,
            ))
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (start_wave, end_wave).run_if(in_state(GameState::Game)),
            );
    }
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::MoveCamera, GamepadStick::LEFT);
        input_map.insert_dual_axis(Self::MovePlaceholderTower, GamepadStick::RIGHT);
        input_map.insert(Self::ToggleTowerType, GamepadButtonType::East);
        input_map.insert(Self::PlaceTower, GamepadButtonType::South);
        input_map.insert(Self::EndPlacement, GamepadButtonType::West);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::MoveCamera, KeyboardVirtualDPad::WASD);
        input_map.insert_dual_axis(Self::MovePlaceholderTower, KeyboardVirtualDPad::ARROW_KEYS);
        input_map.insert(Self::ToggleTowerType, KeyCode::KeyT);
        input_map.insert(Self::PlaceTower, KeyCode::Space);
        input_map.insert(Self::EndPlacement, KeyCode::Enter);

        input_map
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

#[derive(Default, Component)]
struct Wave {
    timer: Timer,
}

fn start_wave(
    action_state: Res<ActionState<PlayerAction>>,
    mut next_state: ResMut<NextState<GamePlayState>>,
    mut commands: Commands,
) {
    if action_state.just_pressed(&PlayerAction::EndPlacement) {
        next_state.set(GamePlayState::Wave);
        commands.spawn((Wave {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
        },));
    }
}

fn end_wave(
    mut next_state: ResMut<NextState<GamePlayState>>,
    time: Res<Time>,
    mut query: Query<&mut Wave>,
) {
    for mut wave in query.iter_mut() {
        wave.timer.tick(time.delta());
        if wave.timer.finished() {
            // && enemies are dead
            next_state.set(GamePlayState::Placement);
        }
    }
}
