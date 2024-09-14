use std::f32::consts::PI;

use super::GameState;
use bevy::color::palettes::css::{ORANGE_RED, WHITE};
use bevy::gltf::Gltf;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    MoveCamera,
    MovePlaceholderTower,
    Place,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::MoveCamera => InputControlKind::DualAxis,
            PlayerAction::MovePlaceholderTower => InputControlKind::DualAxis,
            PlayerAction::Place => InputControlKind::Button,
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "models/sv-charmander.glb")]
    charmander: Handle<Gltf>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<PlayerAction>::default(),))
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(Update, control_camera.run_if(in_state(GameState::Game)))
            .add_systems(
                Update,
                control_placeholder.run_if(in_state(GameState::Game)),
            )
            .add_systems(Update, place_tower.run_if(in_state(GameState::Game)));
    }
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::MoveCamera, GamepadStick::LEFT);
        input_map.insert_dual_axis(Self::MovePlaceholderTower, GamepadStick::RIGHT);
        input_map.insert(Self::Place, GamepadButtonType::South);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::MoveCamera, KeyboardVirtualDPad::WASD);
        input_map.insert_dual_axis(Self::MovePlaceholderTower, KeyboardVirtualDPad::ARROW_KEYS);
        input_map.insert(Self::Place, KeyCode::Space);

        input_map
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Camera;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct TowerPlaceholder;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Tower;

fn setup(mut commands: Commands, assets: Res<GltfAssets>, assets_gltf: Res<Assets<Gltf>>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 5.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        Camera,
    ));

    // spawn spotlights, etc.
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

    if let Some(gltf) = assets_gltf.get(&assets.charmander.clone()) {
        commands.spawn((
            SceneBundle {
                scene: gltf.scenes[0].clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Z, Vec3::Y),
                ..Default::default()
            },
            AsyncSceneCollider {
                shape: Some(ComputedColliderShape::TriMesh),
                ..Default::default()
            },
            TowerPlaceholder,
        ));
    }
}

fn control_camera(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta = time.delta_seconds()
        * action_state
            .clamped_axis_pair(&PlayerAction::MoveCamera)
            .xy();
    player_transform.translation += Vec3::new(move_delta.x, 0.0, -move_delta.y);
}

fn control_placeholder(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<TowerPlaceholder>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta = time.delta_seconds()
        * action_state
            .clamped_axis_pair(&PlayerAction::MovePlaceholderTower)
            .xy();
    player_transform.translation += Vec3::new(move_delta.x, 0.0, -move_delta.y);
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub speed: f32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct ShotTimer {
    timer: Timer,
}

fn place_tower(
    action_state: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
    placeholder_query: Query<&Transform, With<TowerPlaceholder>>,
    assets: Res<GltfAssets>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    let placeholder = placeholder_query.single();
    if action_state.just_pressed(&PlayerAction::Place) {
        if let Some(gltf) = assets_gltf.get(&assets.charmander.clone()) {
            commands.spawn((
                SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    transform: placeholder.looking_to(Vec3::Z, Vec3::Y),
                    ..Default::default()
                },
                AsyncSceneCollider {
                    shape: Some(ComputedColliderShape::TriMesh),
                    ..Default::default()
                },
                Tower,
            ));
        }
    }
}
