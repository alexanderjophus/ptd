mod camera;
mod economy;
mod placement;
mod rolling;
mod wave;

use super::GameState;
use bevy::prelude::*;
use bevy::{
    ecs::system::SystemState, gltf::Gltf, gltf::GltfMesh, math::vec2, render::primitives::Aabb,
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use camera::CameraPlugin;
use economy::EconomyPlugin;
use placement::{CursorPlaceholder, PlacementPlugin};
use rolling::RollingPlugin;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;
use vleue_navigator::prelude::*;
use wave::{Enemy, EnemyDetails, EnemySpawner, WavePlugin};

const SNAP_OFFSET: f32 = 0.5;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePlayState>()
            .add_plugins((
                CameraPlugin,
                EconomyPlugin,
                PlacementPlugin,
                RollingPlugin,
                WavePlugin,
                RonAssetPlugin::<AssetCollections>::new(&["game.ron"]),
                VleueNavigatorPlugin,
                NavmeshUpdaterPlugin::<Aabb, Obstacle>::default(),
            ))
            .init_resource::<Assets<TowerDetails>>()
            .init_resource::<Assets<EnemyDetails>>()
            .init_resource::<DiePool>()
            .add_systems(OnEnter(GameState::Game), setup);
    }
}

// Enum that will be used as a state for the gameplay loop
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GamePlayState {
    #[default]
    Economy,
    Rolling,
    Placement,
    Wave,
}

#[derive(Component, Debug)]
struct Obstacle;

#[derive(Resource, Debug)]
pub struct Resources {
    towers: Vec<AssetId<TowerDetails>>,
    current_tower: usize,
}

impl FromWorld for Resources {
    fn from_world(world: &mut World) -> Self {
        let mut system_state = SystemState::<Res<Assets<TowerDetails>>>::new(world);
        let tower_assets = system_state.get(world);
        let towers = tower_assets.iter().map(|(id, _)| id.clone()).collect();
        Resources {
            towers,
            current_tower: 0,
        }
    }
}

/// Representation of a loaded tower file.
#[derive(Asset, Debug, TypePath, Default)]
pub struct TowerDetails {
    pub name: String,
    pub cost: u32,
    pub range: f32,
    pub damage: u32,
    pub rate_of_fire: f32,
    pub projectile_speed: f32,
    pub model: Handle<Gltf>,
}

#[derive(serde::Deserialize, Debug, Clone)]
enum CustomDynamicAsset {
    Tower {
        name: String,
        cost: u32,
        range: f32,
        damage: u32,
        rate_of_fire: f32,
        projectile_speed: f32,
        model: String,
    },
    Enemy {
        name: String,
        health: u32,
        speed: f32,
        model: String,
    },
}

impl DynamicAsset for CustomDynamicAsset {
    // At this point, the content of your dynamic asset file is done loading.
    // You should return untyped handles to any assets that need to finish loading for your
    // dynamic asset to be ready.
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            CustomDynamicAsset::Tower { model, .. } => {
                vec![asset_server.load::<Gltf>(model).untyped()]
            }
            CustomDynamicAsset::Enemy { model, .. } => {
                vec![asset_server.load::<Gltf>(model).untyped()]
            }
        }
    }

    // This method is called when all asset handles returned from `load` are done loading.
    // The handles that you return, should also be loaded.
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            CustomDynamicAsset::Tower {
                name,
                cost,
                range,
                damage,
                rate_of_fire,
                projectile_speed,
                model,
            } => {
                info!(
                    "Building tower: {} with cost: {}, range: {}, damage: {}, rate_of_fire: {}",
                    name, cost, range, damage, rate_of_fire
                );
                let mut gltf_system_state = SystemState::<Res<AssetServer>>::new(world);
                let asset_server = gltf_system_state.get(world);
                let handle: Handle<Gltf> = asset_server.load(model);

                let mut towers_system_state =
                    SystemState::<ResMut<Assets<TowerDetails>>>::new(world);
                let mut towers = towers_system_state.get_mut(world);
                Ok(DynamicAssetType::Single(
                    towers
                        .add(TowerDetails {
                            name: name.clone(),
                            cost: *cost,
                            range: *range,
                            damage: *damage,
                            rate_of_fire: *rate_of_fire,
                            projectile_speed: *projectile_speed,
                            model: handle,
                        })
                        .untyped(),
                ))
            }
            CustomDynamicAsset::Enemy {
                name,
                health,
                speed,
                model,
            } => {
                info!(
                    "Building enemy: {} with health: {}, speed: {}",
                    name, health, speed
                );
                let mut gltf_system_state = SystemState::<Res<AssetServer>>::new(world);
                let asset_server = gltf_system_state.get(world);
                let handle = asset_server.load(model);

                let mut enemies_system_state =
                    SystemState::<ResMut<Assets<EnemyDetails>>>::new(world);
                let mut enemies = enemies_system_state.get_mut(world);
                Ok(DynamicAssetType::Single(
                    enemies
                        .add(EnemyDetails {
                            name: name.clone(),
                            health: *health,
                            speed: *speed,
                            model: handle,
                        })
                        .untyped(),
                ))
            }
        }
    }
}

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct AssetCollections(HashMap<String, CustomDynamicAsset>);

impl DynamicAssetCollection for AssetCollections {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct TowerAssets {
    #[asset(key = "centaur")]
    pub centaur: Handle<TowerDetails>,
    #[asset(key = "demon")]
    pub demon: Handle<TowerDetails>,
}

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(key = "orc")]
    pub orc: Handle<EnemyDetails>,
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "models/house.glb")]
    pub house: Handle<Gltf>,
}

#[derive(Default, Component)]
struct Goal;

#[derive(Resource)]
struct DieFace {
    primary_type: BaseElementType,
    secondary_type: Option<BaseElementType>,
    rarity: Rarity,
}

#[derive(Component, Debug, Clone, PartialEq)]
enum BaseElementType {
    Fire,  // Heat and destruction
    Water, // Flow and adaptability
    Earth, // Stability and strength
    Wind,  // Movement and agility
}

#[derive(Component, Debug, Clone, PartialEq)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Unique,
}

#[derive(Resource)]
struct Die {
    faces: [DieFace; 6],
    rarity: Rarity,
}

#[derive(Debug, Clone, PartialEq)]
enum DieShopItem {
    TypedDie {
        primary_type: BaseElementType,
        rarity: Rarity,
        cost: usize,
    },
    RandomDie {
        rarity: Rarity,
        cost: usize,
    },
}

#[derive(Resource)]
struct DiePool {
    highlighted: usize,
    items: Vec<DieShopItem>,
}

impl Default for DiePool {
    fn default() -> Self {
        DiePool {
            highlighted: 0,
            items: vec![],
        }
    }
}

impl DiePool {
    fn roll(&mut self) {
        self.highlighted = 0;
        for item in self.items.iter() {
            match item {
                DieShopItem::TypedDie {
                    primary_type,
                    rarity,
                    ..
                } => {
                    info!(
                        "Rolling a die with primary type: {:?} and rarity: {:?}",
                        primary_type, rarity
                    );
                }
                DieShopItem::RandomDie { rarity, .. } => {
                    info!("Rolling a random die with rarity: {:?}", rarity);
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    assets_enemydetails: Res<Assets<EnemyDetails>>,
    enemyassets: Res<EnemyAssets>,
    gltfassets: Res<GltfAssets>,
    res: Res<Assets<Gltf>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        Name::new("Directional Light"),
    ));

    commands.spawn((
        Mesh3d(assets_mesh.add(Circle::new(0.5))),
        Transform::default()
            .with_translation(Vec3::new(SNAP_OFFSET, 0.0, SNAP_OFFSET))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        CursorPlaceholder,
    ));

    let enemy = assets_enemydetails.get(&enemyassets.orc).unwrap();
    let enemy_mesh = res.get(&enemy.model).unwrap();
    let enemy_mesh_mesh = assets_gltfmesh.get(&enemy_mesh.meshes[0]).unwrap();

    commands.spawn((
        Mesh3d(enemy_mesh_mesh.primitives[0].mesh.clone()),
        MeshMaterial3d(enemy_mesh.materials[0].clone()),
        Transform::from_translation(Vec3::new(0.5, 0.0, -10.0)),
        EnemySpawner {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        },
    ));

    let house_mesh = res.get(&gltfassets.house).unwrap();
    let house_mesh_mats = assets_gltfmesh.get(&house_mesh.meshes[0]).unwrap();

    commands.spawn((
        Mesh3d(house_mesh_mats.primitives[0].mesh.clone()),
        MeshMaterial3d(house_mesh.materials[0].clone()),
        Transform::from_translation(Vec3::new(-5.8, 0.0, -4.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.25)),
        Obstacle,
        Name::new("House"),
    ));

    // spawn square placeholder for goal
    commands.spawn((
        Mesh3d(assets_mesh.add(Rectangle::new(0.1, 1.0))),
        Transform::default()
            .with_translation(Vec3::new(-3.9, 0.0, -1.5))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Goal,
    ));

    commands.spawn((
        NavMeshSettings {
            // Define the outer borders of the navmesh.
            fixed: Triangulation::from_outer_edges(&[
                vec2(-20.0, -20.0),
                vec2(20.0, -20.0),
                vec2(20.0, 20.0),
                vec2(-20.0, 20.0),
            ]),
            ..default()
        },
        // Mark it for update as soon as obstacles are changed.
        // Other modes can be debounced or manually triggered.
        NavMeshUpdateMode::Direct,
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

#[derive(Default, Component)]
struct Wave {
    timer: Timer,
}
