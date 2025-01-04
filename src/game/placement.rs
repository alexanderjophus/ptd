use bevy::{gltf::GltfMesh, prelude::*, render::primitives::Aabb};
use leafwing_input_manager::prelude::ActionState;

use crate::GameState;

use super::{GamePlayState, Obstacle, PlayerAction, Resources, TowerDetails, SNAP_OFFSET};

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (
                    control_cursor,
                    placeholder_snap_to_cursor,
                    toggle_placeholder_type,
                    place_tower,
                )
                    .run_if(in_state(GameState::Game).and(in_state(GamePlayState::Placement))),
            );
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    pub name: String,
    pub cost: u32,
    pub range: f32,
    pub damage: u32,
    pub projectile_speed: f32,
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

fn setup(
    mut commands: Commands,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_towers: Res<Assets<TowerDetails>>,
    res: Res<Assets<Gltf>>,
    tower: Res<Resources>,
) {
    // spawn tower placeholder
    let tower = assets_towers
        .get(tower.towers[tower.current_tower])
        .unwrap();
    let tower_mesh = res.get(&tower.model).unwrap();
    let tower_mesh_mesh = assets_gltfmesh.get(&tower_mesh.meshes[0]).unwrap();

    commands.spawn((
        Mesh3d(tower_mesh_mesh.primitives[0].mesh.clone()),
        MeshMaterial3d(tower_mesh.materials[0].clone()),
        Transform::from_scale(Vec3::splat(0.5)),
        TowerPlaceholder,
    ));
}

fn control_cursor(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<CursorPlaceholder>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta = time.delta_secs()
        * 2.0
        * action_state
            .clamped_axis_pair(&PlayerAction::MoveCursorPlaceholder)
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

    let mut placeholder_transform = placeholder_query.single_mut();
    let mut placeholder_position = placeholder_transform.translation;

    let snap_distance = 1.0;
    let snap_x = (cursor_position.x - SNAP_OFFSET / snap_distance).round() + SNAP_OFFSET;
    let snap_z = (cursor_position.z - SNAP_OFFSET / snap_distance).round() + SNAP_OFFSET;

    placeholder_position.x = snap_x;
    placeholder_position.z = snap_z;

    placeholder_transform.translation = placeholder_position;
}

fn toggle_placeholder_type(
    action_state: Res<ActionState<PlayerAction>>,
    mut current_tower: ResMut<Resources>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_towers: Res<Assets<TowerDetails>>,
    res: Res<Assets<Gltf>>,
    // mut query: Query<(&mut Mesh3d, &mut Handle<StandardMaterial>), With<TowerPlaceholder>>,
    mut query: Query<(&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>), With<TowerPlaceholder>>,
) {
    if action_state.just_pressed(&PlayerAction::ToggleTowerType) {
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
    action_state: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
    assets_towers: Res<Assets<TowerDetails>>,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    res: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    current_tower: Res<Resources>,
    placeholder_query: Query<&Transform, With<TowerPlaceholder>>,
) {
    let placeholder_transform = placeholder_query.single();
    if action_state.just_pressed(&PlayerAction::PlaceTower) {
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
                placeholder_transform.clone(),
                Tower {
                    name: placeholder_tower.name.clone(),
                    cost: placeholder_tower.cost,
                    range: placeholder_tower.range,
                    damage: placeholder_tower.damage,
                    projectile_speed: placeholder_tower.projectile_speed,
                    attack_speed: Timer::from_seconds(
                        1. / placeholder_tower.rate_of_fire,
                        TimerMode::Repeating,
                    ),
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
