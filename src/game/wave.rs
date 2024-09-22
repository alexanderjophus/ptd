use std::time::Duration;

use bevy::{gltf::GltfMesh, prelude::*};

use crate::GameState;

use super::{GamePlayState, GltfAssets};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), spawn_enemy_spawner)
            .add_systems(
                Update,
                (spawn_enemy, move_enemy).run_if(in_state(GamePlayState::Wave)),
            );
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct EnemySpawner {
    timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Enemy {
    health: u32,
    speed: f32,
}

fn spawn_enemy_spawner(
    mut commands: Commands,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<GltfAssets>,
    res: Res<Assets<Gltf>>,
) {
    let enemy_mesh = res.get(&assets_gltf.diglett).unwrap();
    let enemy_mesh_mesh = assets_gltfmesh.get(&enemy_mesh.meshes[0]).unwrap();
    commands.spawn((
        PbrBundle {
            mesh: enemy_mesh_mesh.primitives[0].mesh.clone(),
            material: enemy_mesh.materials[0].clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0))
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(4.0)),
            ..Default::default()
        },
        EnemySpawner {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        },
    ));
}

fn spawn_enemy(
    mut commands: Commands,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<GltfAssets>,
    res: Res<Assets<Gltf>>,
    time: Res<Time>,
    mut query: Query<(&mut EnemySpawner, &Transform)>,
) {
    for (mut spawner, transform) in query.iter_mut() {
        spawner.timer.tick(time.delta());
        if spawner.timer.finished() {
            let enemy_mesh = res.get(&assets_gltf.diglett).unwrap();
            let enemy_mesh_mesh = assets_gltfmesh.get(&enemy_mesh.meshes[0]).unwrap();
            commands.spawn((
                PbrBundle {
                    mesh: enemy_mesh_mesh.primitives[0].mesh.clone(),
                    material: enemy_mesh.materials[0].clone(),
                    transform: transform.with_scale(Vec3::splat(1.0)),
                    ..Default::default()
                },
                Enemy {
                    health: 100,
                    speed: 0.01,
                },
            ));
        }
    }
}

fn move_enemy(mut query: Query<(&Enemy, &mut Transform)>) {
    for (enemy, mut transform) in query.iter_mut() {
        transform.translation.z += enemy.speed;
        // base rotate off of z translation
        transform.rotation = Quat::from_rotation_z((transform.translation.z * 2.0).sin() / 2.0)
            .mul_quat(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2));
    }
}
