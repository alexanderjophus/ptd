mod camera;

use std::f32::consts::PI;

use super::GameState;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use camera::CameraPlugin;
use leafwing_input_manager::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    MoveCamera,
    MovePlaceholderTower,
    ToggleTowerType,
    Place,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::MoveCamera => InputControlKind::DualAxis,
            PlayerAction::MovePlaceholderTower => InputControlKind::DualAxis,
            PlayerAction::ToggleTowerType => InputControlKind::Button,
            PlayerAction::Place => InputControlKind::Button,
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "models/sv-charmander.glb")]
    pub charmander: Handle<Gltf>,
    #[asset(path = "models/sv-gastly.glb")]
    pub gastly: Handle<Gltf>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
enum TowerOptions {
    #[default]
    Charmander,
    Gastly,
}

#[derive(Resource, Default)]
pub struct CurrentTower {
    tower_option: TowerOptions,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<PlayerAction>::default(), CameraPlugin))
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .insert_resource(CurrentTower::default())
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (control_placeholder, toggle_type, place_tower).run_if(in_state(GameState::Game)),
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
        input_map.insert(Self::Place, GamepadButtonType::South);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::MoveCamera, KeyboardVirtualDPad::WASD);
        input_map.insert_dual_axis(Self::MovePlaceholderTower, KeyboardVirtualDPad::ARROW_KEYS);
        input_map.insert(Self::ToggleTowerType, KeyCode::KeyT);
        input_map.insert(Self::Place, KeyCode::Space);

        input_map
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct TowerPlaceholder;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Tower;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
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

    // spawn square placeholder for tower
    let placeholder_mesh = meshes.add(Circle::new(1.0));
    commands.spawn((
        PbrBundle {
            mesh: placeholder_mesh,
            transform: Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            )),
            ..default()
        },
        TowerPlaceholder,
    ));
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

fn toggle_type(
    action_state: Res<ActionState<PlayerAction>>,
    mut current_tower: ResMut<CurrentTower>,
) {
    if action_state.just_pressed(&PlayerAction::ToggleTowerType) {
        info!("Toggling tower type");
        current_tower.tower_option = match current_tower.tower_option {
            TowerOptions::Charmander => TowerOptions::Gastly,
            TowerOptions::Gastly => TowerOptions::Charmander,
        };
    }
}

fn place_tower(
    action_state: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
    placeholder_query: Query<&Transform, With<TowerPlaceholder>>,
    assets: Res<GltfAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    current_tower: Res<CurrentTower>,
) {
    let placeholder = placeholder_query.single();
    if action_state.just_pressed(&PlayerAction::Place) {
        let chosen_tower = match current_tower.tower_option {
            TowerOptions::Charmander => &assets.charmander,
            TowerOptions::Gastly => &assets.gastly,
        };
        if let Some(gltf) = assets_gltf.get(chosen_tower) {
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
