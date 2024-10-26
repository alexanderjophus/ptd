use std::time::Duration;

use bevy::{gltf::GltfMesh, prelude::*};
use vleue_navigator::prelude::*;

use crate::GameState;

use super::{
    placement::{Projectile, Tower},
    EnemyAssets, GamePlayState, Goal,
};

pub struct WavePlugin;

impl Plugin for WavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_enemy,
                find_path,
                move_enemy,
                tower_shooting,
                move_projectile,
                bullet_despawn,
                bullet_collision,
                target_death,
                enemy_goal_collision,
            )
                .run_if(in_state(GameState::Game).and_then(in_state(GamePlayState::Wave))),
        );
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct EnemySpawner {
    pub timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Enemy {
    name: String,
    health: u32,
    speed: f32,
}

/// Representation of a loaded enemy file.
#[derive(Asset, Debug, TypePath, Component)]
pub struct EnemyDetails {
    pub name: String,
    pub health: u32,
    pub speed: f32,
    pub model: Handle<Gltf>,
}

fn spawn_enemy(
    mut commands: Commands,
    assets_enemies: Res<Assets<EnemyDetails>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltf: Res<EnemyAssets>,
    res: Res<Assets<Gltf>>,
    time: Res<Time>,
    mut query: Query<(&mut EnemySpawner, &Transform)>,
) {
    for (mut spawner, transform) in query.iter_mut() {
        spawner.timer.tick(time.delta());
        if spawner.timer.finished() {
            let enemy = assets_enemies.get(&assets_gltf.orc).unwrap();
            let enemy_mesh = res.get(&enemy.model).unwrap();
            let enemy_mesh_mesh = assets_gltfmesh.get(&enemy_mesh.meshes[0]).unwrap();

            commands.spawn((
                PbrBundle {
                    mesh: enemy_mesh_mesh.primitives[0].mesh.clone(),
                    material: enemy_mesh.materials[0].clone(),
                    transform: transform.with_scale(Vec3::splat(0.5)),
                    ..Default::default()
                },
                Enemy {
                    name: enemy.name.clone(),
                    health: enemy.health,
                    speed: enemy.speed,
                },
            ));
        }
    }
}

pub fn find_path(
    mut navmeshes: ResMut<Assets<NavMesh>>,
    navmesh: Query<(&Handle<NavMesh>, &NavMeshStatus)>,
    mut from_query: Query<&mut Transform, With<Enemy>>,
    to_query: Query<&Transform, (With<Goal>, Without<Enemy>)>,
) {
    let (navmesh_handle, status) = navmesh.single();
    if *status != NavMeshStatus::Built {
        return;
    }
    if let Some(navmesh) = navmeshes.get_mut(navmesh_handle) {
        let to = to_query.single().translation;
        from_query.iter_mut().for_each(|mut from| {
            if let Some(path) = navmesh.transformed_path(from.translation, to) {
                let next = path.path[0];
                from.look_at(Vec3::new(next.x, next.y, next.z), Vec3::Y);
            } else {
                warn_once!("no path found from {:?} to {:?}", from, to);
            }
        });
    }
}

fn move_enemy(mut query: Query<&mut Transform, With<Enemy>>) {
    for mut transform in query.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 0.01;
        // base rotate off of z translation
        transform.rotation = Quat::from_rotation_z((transform.translation.z * 8.0).sin() * 0.1);
    }
}

fn tower_shooting(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
    mut query_tower: Query<(&Transform, &mut Tower)>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for (enemy, enemy_transform) in query.iter() {
        for (tower_transform, mut tower) in query_tower.iter_mut() {
            tower.attack_speed.tick(time.delta());
            if tower.attack_speed.finished() {
                let bullet_spawn = tower_transform.translation; //  + tower.bullet_offset;

                let distance = tower_transform
                    .translation
                    .distance(enemy_transform.translation);

                let placeholder_mesh = meshes.add(Sphere::new(0.1));
                if distance < tower.range {
                    commands.spawn((
                        PbrBundle {
                            mesh: placeholder_mesh.clone(),
                            transform: Transform::from_translation(bullet_spawn),
                            ..Default::default()
                        },
                        Projectile {
                            target: enemy,
                            speed: tower.projectile_speed,
                            damage: tower.damage,
                            lifetime: Timer::new(Duration::from_secs(1), TimerMode::Once),
                        },
                    ));
                    tower.attack_speed.reset();
                }
            }
        }
    }
}

fn move_projectile(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Projectile), Without<Enemy>>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for (entity, mut transform, projectile) in query.iter_mut() {
        if let Ok(target) = enemy_query.get(projectile.target) {
            let direction = target.translation - transform.translation;
            let distance = direction.length();
            let velocity = direction.normalize() * projectile.speed * time.delta_seconds();

            if distance < velocity.length() {
                commands.entity(entity).despawn();
            } else {
                transform.translation += velocity;
            }
        }
    }
}

fn bullet_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in &mut bullets {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &GlobalTransform, &Projectile), With<Projectile>>,
    mut targets: Query<(&mut Enemy, &Transform), With<Enemy>>,
) {
    for (bullet, bullet_transform, projectile) in &bullets {
        for (mut enemy, target_transform) in &mut targets {
            if Vec3::distance(bullet_transform.translation(), target_transform.translation) < 0.4 {
                commands.entity(bullet).despawn_recursive();
                enemy.health = enemy
                    .health
                    .checked_sub(projectile.damage)
                    .unwrap_or_default();
                break;
            }
        }
    }
}

fn enemy_goal_collision(
    mut commands: Commands,
    goals: Query<&Transform, With<Goal>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for goal_transform in &goals {
        for (entity, enemy_transform) in &enemies {
            if Vec3::distance(goal_transform.translation, enemy_transform.translation) < 0.4 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn target_death(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy)>,
    projectiles: Query<(Entity, &Projectile)>,
) {
    for (ent, enemy) in &enemies {
        if enemy.health <= 0 {
            commands.entity(ent).despawn_recursive();
        }
    }
    for (ent, projectile) in &projectiles {
        if let Err(_) = enemies.get(projectile.target) {
            commands.entity(ent).despawn_recursive();
        }
    }
}
