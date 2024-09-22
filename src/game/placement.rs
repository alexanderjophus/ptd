use std::f32::consts::PI;

use bevy::{gltf::GltfMesh, prelude::*};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::GameState;

use super::{GamePlayState, GltfAssets, PlayerAction};

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentTower::default())
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (control_placeholder, toggle_placeholder_type, place_tower)
                    .run_if(in_state(GameState::Game).and_then(in_state(GamePlayState::Placement))),
            );
    }
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

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Tower;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct TowerPlaceholder;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<GltfAssets>,
    res: Res<Assets<Gltf>>,
    current_tower: Res<CurrentTower>,
) {
    // spawn square placeholder for tower
    let placeholder_mesh = meshes.add(Circle::new(1.0));
    let placeholder_tower = match current_tower.tower_option {
        TowerOptions::Charmander => res.get(&assets_gltf.charmander).unwrap(),
        TowerOptions::Gastly => res.get(&assets_gltf.gastly).unwrap(),
    };
    let placeholder_tower_mesh = assets_gltfmesh.get(&placeholder_tower.meshes[0]).unwrap();
    commands
        .spawn((
            PbrBundle {
                mesh: placeholder_tower_mesh.primitives[0].mesh.clone(),
                material: placeholder_tower.materials[0].clone(),
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
    mut current_tower: ResMut<CurrentTower>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<GltfAssets>,
    res: Res<Assets<Gltf>>,
    mut query: Query<(&mut Handle<Mesh>, &mut Handle<StandardMaterial>), With<TowerPlaceholder>>,
) {
    if action_state.just_pressed(&PlayerAction::ToggleTowerType) {
        current_tower.tower_option = match current_tower.tower_option {
            TowerOptions::Charmander => TowerOptions::Gastly,
            TowerOptions::Gastly => TowerOptions::Charmander,
        };
        let placeholder_tower = match current_tower.tower_option {
            TowerOptions::Charmander => res.get(&assets_gltf.charmander).unwrap(),
            TowerOptions::Gastly => res.get(&assets_gltf.gastly).unwrap(),
        };
        let placeholder_tower_mesh = assets_gltfmesh.get(&placeholder_tower.meshes[0]).unwrap();
        let (mut mesh, mut mat) = query.single_mut();
        *mesh = placeholder_tower_mesh.primitives[0].mesh.clone();
        *mat = placeholder_tower.materials[0].clone();
    }
}

fn place_tower(
    action_state: Res<ActionState<PlayerAction>>,
    mut commands: Commands,
    res: Res<Assets<Gltf>>,
    assets_gltf: Res<GltfAssets>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    current_tower: Res<CurrentTower>,
    placeholder_query: Query<&Transform, With<TowerPlaceholder>>,
) {
    let placeholder_transform = placeholder_query.single();
    if action_state.just_pressed(&PlayerAction::PlaceTower) {
        let placeholder_tower = match current_tower.tower_option {
            TowerOptions::Charmander => res.get(&assets_gltf.charmander).unwrap(),
            TowerOptions::Gastly => res.get(&assets_gltf.gastly).unwrap(),
        };
        let placeholder_tower_mesh = assets_gltfmesh.get(&placeholder_tower.meshes[0]).unwrap();

        commands.spawn((
            PbrBundle {
                mesh: placeholder_tower_mesh.primitives[0].mesh.clone(),
                material: placeholder_tower.materials[0].clone(),
                transform: placeholder_transform.clone(),
                ..default()
            },
            AsyncSceneCollider {
                shape: Some(ComputedColliderShape::TriMesh),
                ..Default::default()
            },
            Tower,
        ));
    }
}
