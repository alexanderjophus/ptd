use std::f32::consts::PI;

use bevy::{gltf::GltfMesh, prelude::*};
use leafwing_input_manager::prelude::ActionState;

use crate::GameState;

use super::{GamePlayState, PlayerAction, Resources, TowerDetails};

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (control_placeholder, toggle_placeholder_type, place_tower)
                    .run_if(in_state(GameState::Game).and_then(in_state(GamePlayState::Placement))),
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
struct TowerPlaceholder;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_towers: Res<Assets<TowerDetails>>,
    res: Res<Assets<Gltf>>,
    tower: Res<Resources>,
) {
    // spawn circle placeholder for tower
    let placeholder_mesh = meshes.add(Circle::new(1.0));

    // spawn tower placeholder
    let tower = assets_towers
        .get(tower.towers[tower.current_tower])
        .unwrap();
    let tower_mesh = res.get(&tower.model).unwrap();
    let tower_mesh_mesh = assets_gltfmesh.get(&tower_mesh.meshes[0]).unwrap();

    commands
        .spawn((
            PbrBundle {
                mesh: tower_mesh_mesh.primitives[0].mesh.clone(),
                material: tower_mesh.materials[0].clone(),
                transform: Transform::default().with_rotation(
                    Quat::from_rotation_x(PI / 2.).mul_quat(Quat::from_rotation_z(PI)),
                ),
                ..default()
            },
            TowerPlaceholder,
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: placeholder_mesh,
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            });
        });
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

fn toggle_placeholder_type(
    action_state: Res<ActionState<PlayerAction>>,
    mut current_tower: ResMut<Resources>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_towers: Res<Assets<TowerDetails>>,
    res: Res<Assets<Gltf>>,
    mut query: Query<(&mut Handle<Mesh>, &mut Handle<StandardMaterial>), With<TowerPlaceholder>>,
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
        *mesh = placeholder_tower_mesh.primitives[0].mesh.clone();
        *mat = placeholder_tower_gltf.materials[0].clone();
    }
}

fn place_tower(
    action_state: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
    assets_towers: Res<Assets<TowerDetails>>,
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

        commands.spawn((
            PbrBundle {
                mesh: tower_mesh_mesh.primitives[0].mesh.clone(),
                material: tower_mesh.materials[0].clone(),
                transform: placeholder_transform.clone(),
                ..default()
            },
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
        ));
    }
}
