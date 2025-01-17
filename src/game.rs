mod camera;
mod economy;
mod placement;
mod roll;
mod wave;

use super::GameState;
use bevy::gltf::GltfMesh;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::{ecs::system::SystemState, gltf::Gltf, render::primitives::Aabb};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use camera::CameraPlugin;
use economy::EconomyPlugin;
use placement::PlacementPlugin;
use rand::seq::IteratorRandom;
use rand::Rng;
use roll::RollPlugin;
use std::f32::consts::PI;
use std::time::Duration;
use vleue_navigator::prelude::*;
use wave::{EnemySpawner, WavePlugin};

const SNAP_OFFSET: f32 = 0.5;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GamePlayState>()
            .add_plugins((
                CameraPlugin,
                EconomyPlugin,
                PlacementPlugin,
                RollPlugin,
                WavePlugin,
                RonAssetPlugin::<AssetCollections>::new(&["game.ron"]),
                VleueNavigatorPlugin,
                NavmeshUpdaterPlugin::<Aabb, Obstacle>::default(),
            ))
            .init_resource::<Assets<TowerDetails>>()
            .init_resource::<Assets<EnemyDetails>>()
            .init_resource::<DiePool>()
            .init_resource::<TowerPool>()
            .insert_resource(DiePool {
                dice: Vec::new(),
                highlighted: 0,
            })
            .insert_resource(TowerPool {
                towers: Vec::new(),
                highlighted: 0,
            })
            .register_type::<DiePool>()
            .add_event::<DiePurchaseEvent>()
            .add_event::<DieRolledEvent>()
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (die_purchased, die_rolled).run_if(in_state(GameState::Game)),
            );
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

#[derive(AssetCollection, Resource)]
pub struct AllAssets {
    #[asset(key = "towers", collection(typed))]
    pub towers: Vec<Handle<TowerDetails>>,
    #[asset(key = "enemies", collection(typed))]
    pub enemies: Vec<Handle<EnemyDetails>>,
}

/// Representation of a loaded tower file.
#[derive(Asset, Resource, Debug, PartialEq, Clone, TypePath)]
pub struct TowerDetails {
    pub name: String,
    pub cost: u32,
    pub element_type: BaseElementType,
    pub model: Handle<Gltf>,
}

/// Representation of a loaded enemy file.
#[derive(Asset, Debug, TypePath)]
pub struct EnemyDetails {
    pub name: String,
    pub health: u32,
    pub speed: f32,
    pub model: Handle<Gltf>,
}

#[derive(serde::Deserialize, Debug, Clone)]
enum CustomDynamicAsset {
    Towers(Vec<TowerDetailsRon>),
    Enemies(Vec<EnemyDetailsRon>),
}

impl DynamicAsset for CustomDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            CustomDynamicAsset::Towers(towers) => towers
                .iter()
                .map(|tower| asset_server.load::<Gltf>(tower.model.clone()).untyped())
                .collect(),
            CustomDynamicAsset::Enemies(enemies) => enemies
                .iter()
                .map(|enemy| asset_server.load::<Gltf>(enemy.model.clone()).untyped())
                .collect(),
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            CustomDynamicAsset::Towers(towers) => {
                let mut towers_collection = vec![];
                for tower in towers {
                    let model = world
                        .get_resource::<AssetServer>()
                        .unwrap()
                        .load(tower.model.clone());
                    let mut tower_details =
                        SystemState::<ResMut<Assets<TowerDetails>>>::new(world).get_mut(world);
                    let handle = tower_details.add(TowerDetails {
                        name: tower.name.clone(),
                        cost: tower.cost,
                        element_type: tower.element_type.clone(),
                        model: model.clone(),
                    });
                    towers_collection.push(handle.untyped());
                    info!("Built tower: {}", tower.name);
                }
                Ok(DynamicAssetType::Collection(towers_collection))
            }
            CustomDynamicAsset::Enemies(enemies) => {
                let mut enemies_collection = vec![];
                for enemy in enemies {
                    let model = world
                        .get_resource::<AssetServer>()
                        .unwrap()
                        .load(enemy.model.clone());
                    let mut assets = world.get_resource_mut::<Assets<EnemyDetails>>().unwrap();
                    let handle = assets.add(EnemyDetails {
                        name: enemy.name.clone(),
                        health: enemy.health,
                        speed: enemy.speed,
                        model: model.clone(),
                    });
                    enemies_collection.push(handle.untyped());
                    info!("Built enemy: {}", enemy.name);
                }
                Ok(DynamicAssetType::Collection(enemies_collection))
            }
        }
    }
}

#[derive(serde::Deserialize, Asset, Debug, TypePath, Clone)]
pub struct TowerDetailsRon {
    pub name: String,
    pub cost: u32,
    pub element_type: BaseElementType,
    pub model: String,
}

#[derive(serde::Deserialize, Asset, Debug, TypePath, Clone)]
pub struct EnemyDetailsRon {
    pub name: String,
    pub health: u32,
    pub speed: f32,
    pub model: String,
}

#[derive(AssetCollection, Resource)]
pub struct GltfAssets {
    #[asset(path = "models/house.glb")]
    pub house: Handle<Gltf>,
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

#[derive(Default, Component)]
struct Goal;

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct DieFace {
    primary_type: BaseElementType,
    rarity: Rarity,
}

impl std::fmt::Display for DieFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write the primary type
        write!(
            f,
            "{}",
            match self.primary_type {
                BaseElementType::None => "None",
                BaseElementType::Fire => "Fire",
                BaseElementType::Water => "Water",
                BaseElementType::Earth => "Earth",
                BaseElementType::Wind => "Wind",
            }
        )
    }
}

#[derive(Resource, serde::Deserialize, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub enum BaseElementType {
    #[default]
    None, // No element
    Fire,  // Heat and destruction
    Water, // Flow and adaptability
    Earth, // Stability and strength
    Wind,  // Movement and agility
}

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Unique,
}

#[derive(Event)]
struct DiePurchaseEvent(Die);

#[derive(Event)]
struct DieRolledEvent(DieFace);

#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct Die {
    // the faces of the die
    faces: [DieFace; 6],
    // the current monetary value of the die
    value: usize,
}

impl Die {
    fn roll(&self) -> DieFace {
        let mut rng = rand::thread_rng();
        let face = rng.gen_range(0..6);
        self.faces[face].clone()
    }
}

impl std::fmt::Display for Die {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Die with faces: {},{},{},{},{},{}",
            self.faces[0],
            self.faces[1],
            self.faces[2],
            self.faces[3],
            self.faces[4],
            self.faces[5]
        )
    }
}

struct DieBuilder {
    faces: [DieFace; 6],
}

impl DieBuilder {
    pub fn from_type(selected_type: BaseElementType) -> Self {
        let face = DieFace {
            primary_type: selected_type,
            rarity: Rarity::Common, // todo: make this random
        };
        DieBuilder {
            faces: [
                face.clone(),
                face.clone(),
                face.clone(),
                face.clone(),
                face.clone(),
                face,
            ],
        }
    }

    fn build(self) -> Die {
        Die {
            faces: self.faces,
            value: 10,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct DiePool {
    dice: Vec<Die>,
    highlighted: usize,
}

impl DiePool {
    // remove the die from the pool and return the rolled face
    fn roll(&mut self) -> DieFace {
        let idx = self.highlighted;
        let die = self.dice.remove(idx);
        die.roll()
    }
}

#[derive(Resource, Default, Debug, PartialEq)]
struct TowerPool {
    towers: Vec<AssetId<TowerDetails>>,
    highlighted: usize,
}

impl TowerPool {
    fn toggle_highlighted(&mut self) {
        if self.towers.is_empty() {
            return;
        }
        self.highlighted = (self.highlighted + 1) % self.towers.len();
    }
}

fn setup(
    mut commands: Commands,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    assets_enemydetails: Res<Assets<EnemyDetails>>,
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

    // get first enemy from assets
    let enemy = assets_enemydetails.iter().next().unwrap();
    let enemy_mesh = res.get(&enemy.1.model).unwrap();
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

fn die_purchased(mut die_pool: ResMut<DiePool>, mut ev_purchased: EventReader<DiePurchaseEvent>) {
    for ev in ev_purchased.read() {
        die_pool.dice.push(ev.0.clone());
    }
}

fn die_rolled(
    tower_assets: Res<Assets<TowerDetails>>,
    mut tower_pool: ResMut<TowerPool>,
    mut ev_rolled: EventReader<DieRolledEvent>,
) {
    for ev in ev_rolled.read() {
        let face = ev.0.clone();
        let selected_type = face.primary_type.clone();
        let (id, tower) = tower_assets
            .iter()
            .filter(|(_, tower)| tower.element_type == selected_type)
            .choose(&mut rand::thread_rng())
            .unwrap();
        info!("Selected tower: {}", tower.name);
        tower_pool.towers.push(id);
    }
}
