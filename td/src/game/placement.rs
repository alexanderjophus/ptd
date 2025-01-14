use bevy::{gltf::GltfMesh, prelude::*, render::primitives::Aabb};
use leafwing_input_manager::{prelude::*, Actionlike, InputControlKind};

use crate::GameState;

use super::{BaseElementType, GamePlayState, Obstacle, Resources, TowerDetails, Wave, SNAP_OFFSET};

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlacementAction>::default())
            .init_resource::<ActionState<PlacementAction>>()
            .insert_resource(PlacementAction::default_input_map())
            .add_systems(
                Update,
                (
                    control_cursor,
                    placeholder_snap_to_cursor,
                    toggle_placeholder_type,
                    place_tower,
                    start_wave,
                )
                    .run_if(in_state(GameState::Game).and(in_state(GamePlayState::Placement))),
            );
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlacementAction {
    MoveCursorPlaceholder,
    ToggleTowerType,
    PlaceTower,
    EndPlacement,
}

impl Actionlike for PlacementAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlacementAction::MoveCursorPlaceholder => InputControlKind::DualAxis,
            PlacementAction::ToggleTowerType => InputControlKind::Button,
            PlacementAction::PlaceTower => InputControlKind::Button,
            PlacementAction::EndPlacement => InputControlKind::Button,
        }
    }
}

impl PlacementAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::MoveCursorPlaceholder, GamepadStick::RIGHT);
        input_map.insert(Self::ToggleTowerType, GamepadButton::East);
        input_map.insert(Self::PlaceTower, GamepadButton::South);
        input_map.insert(Self::EndPlacement, GamepadButton::West);

        // // Default kbm input bindings
        input_map.insert_dual_axis(Self::MoveCursorPlaceholder, VirtualDPad::arrow_keys());
        input_map.insert(Self::ToggleTowerType, KeyCode::KeyT);
        input_map.insert(Self::PlaceTower, KeyCode::Space);
        input_map.insert(Self::EndPlacement, KeyCode::Enter);

        input_map
    }
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Tower {
    pub name: String,
    pub cost: u32,
    pub r#type: BaseElementType,
    pub attack_speed: Timer,
}

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Projectile {
    pub speed: f32,
    pub damage: u32,
    pub target: Entity,
    pub lifetime: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct TowerPlaceholder;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct CursorPlaceholder;

fn control_cursor(
    time: Res<Time>,
    action_state: Res<ActionState<PlacementAction>>,
    mut query: Query<&mut Transform, With<CursorPlaceholder>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta = time.delta_secs()
        * 2.0
        * action_state
            .clamped_axis_pair(&PlacementAction::MoveCursorPlaceholder)
            .xy();
    player_transform.translation += Vec3::new(move_delta.x, 0.0, -move_delta.y);
}

// snaps the tower placeholder to the nearest spot on the grid to the cursor
fn placeholder_snap_to_cursor(
    mut placeholder_query: Query<
        &mut Transform,
        (With<TowerPlaceholder>, Without<CursorPlaceholder>),
    >,
    cursor_query: Query<&mut Transform, (With<CursorPlaceholder>, Without<TowerPlaceholder>)>,
) {
    let cursor_transform = cursor_query.single();
    let cursor_position = cursor_transform.translation;

    placeholder_query
        .iter_mut()
        .for_each(|mut placeholder_transform| {
            let snap_distance = 1.0;
            let snap_x = (cursor_position.x - SNAP_OFFSET / snap_distance).round() + SNAP_OFFSET;
            let snap_z = (cursor_position.z - SNAP_OFFSET / snap_distance).round() + SNAP_OFFSET;

            placeholder_transform.translation.x = snap_x;
            placeholder_transform.translation.z = snap_z;
        });
}

fn toggle_placeholder_type(
    action_state: Res<ActionState<PlacementAction>>,
    mut current_tower: ResMut<Resources>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_towers: Res<Assets<TowerDetails>>,
    res: Res<Assets<Gltf>>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>), With<TowerPlaceholder>>,
) {
    if action_state.just_pressed(&PlacementAction::ToggleTowerType) {
        current_tower.current_tower =
            (current_tower.current_tower + 1) % current_tower.towers.len();

        let placeholder_tower_id = current_tower.towers[current_tower.current_tower];
        let placeholder_tower = &assets_towers.get(placeholder_tower_id).unwrap().model;
        let placeholder_tower_gltf = res.get(placeholder_tower).unwrap();
        let placeholder_tower_mesh = assets_gltfmesh
            .get(&placeholder_tower_gltf.meshes[0])
            .unwrap();
        let (mut mesh, mut mat) = query.single_mut();
        mesh.0 = placeholder_tower_mesh.primitives[0].mesh.clone();
        mat.0 = placeholder_tower_gltf.materials[0].clone();
    }
}

fn place_tower(
    action_state: Res<ActionState<PlacementAction>>,
    mut commands: Commands,
    assets_towers: Res<Assets<TowerDetails>>,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    res: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    current_tower: Res<Resources>,
    placeholder_query: Query<&Transform, With<TowerPlaceholder>>,
) {
    if action_state.just_pressed(&PlacementAction::PlaceTower) {
        let placeholder_transform = placeholder_query.single(); // known bug: will panic if player places tower but none selected
        let placeholder_tower = assets_towers
            .get(current_tower.towers[current_tower.current_tower])
            .unwrap();
        let tower_mesh = res.get(&placeholder_tower.model).unwrap();
        let tower_mesh_mesh = assets_gltfmesh.get(&tower_mesh.meshes[0]).unwrap();

        // maybe can be rectangle
        let obstacle_mesh = assets_mesh.add(Cuboid::new(2.0, 2.0, 1.0));
        commands
            .spawn((
                Mesh3d(tower_mesh_mesh.primitives[0].mesh.clone()),
                MeshMaterial3d(tower_mesh.materials[0].clone()),
                *placeholder_transform,
                Tower {
                    name: placeholder_tower.name.clone(),
                    cost: placeholder_tower.cost,
                    r#type: placeholder_tower.element_type.clone(),
                    attack_speed: Timer::from_seconds(1.0, TimerMode::Repeating), // todo
                },
                Obstacle,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(obstacle_mesh.clone()),
                    Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                        .with_translation(Vec3::new(0.0, 0.0, -0.5)),
                    Visibility::Hidden,
                    Aabb::from_min_max(Vec3::ZERO, Vec3::ONE * 2.0),
                    Obstacle,
                ));
            });
    }
}

fn start_wave(
    action_state: Res<ActionState<PlacementAction>>,
    mut next_state: ResMut<NextState<GamePlayState>>,
    mut commands: Commands,
) {
    if action_state.just_pressed(&PlacementAction::EndPlacement) {
        next_state.set(GamePlayState::Wave);
        commands.spawn(Wave {
            timer: Timer::from_seconds(20.0, TimerMode::Once),
        });
    }
}
