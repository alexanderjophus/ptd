use std::time::Duration;

use super::GameState;
use bevy::color::palettes::css::WHITE;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameplayState {
    #[default]
    Playing,
    Paused,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    Pause,
    Move,
    Shoot,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::Pause => InputControlKind::Button,
            PlayerAction::Move => InputControlKind::DualAxis,
            PlayerAction::Shoot => InputControlKind::Button,
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
            .init_state::<GameplayState>()
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::default_input_map())
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(Update, toggle_pause.run_if(in_state(GameState::Game)))
            .add_systems(Update, control_camera.run_if(in_state(GameState::Game)))
            .add_systems(Update, shoot.run_if(in_state(GameState::Game)))
            .add_systems(Update, move_bullets.run_if(in_state(GameState::Game)))
            .add_systems(Update, despawn_shots.run_if(in_state(GameState::Game)));
    }
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::Move, GamepadStick::LEFT);
        input_map.insert(Self::Pause, GamepadButtonType::Start);
        input_map.insert(Self::Shoot, GamepadButtonType::South);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::Move, KeyboardVirtualDPad::WASD);
        input_map.insert(Self::Pause, KeyCode::Escape);
        input_map.insert(Self::Shoot, KeyCode::Space);

        input_map
    }
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Camera;

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

    if let Some(gltf) = assets_gltf.get(&assets.charmander.clone()) {
        commands
            .spawn((
                SceneBundle {
                    scene: gltf.scenes[0].clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Z, Vec3::Y),
                    ..Default::default()
                },
                AsyncSceneCollider {
                    shape: Some(ComputedColliderShape::TriMesh),
                    ..Default::default()
                },
                Tower,
            ))
            .with_children(|parent| {
                parent.spawn(SpotLightBundle {
                    transform: Transform::from_xyz(-1.0, 2.0, 0.0)
                        .looking_at(Vec3::new(-1.0, 0.0, 0.0), Vec3::Z),
                    spot_light: SpotLight {
                        intensity: 100_000.0,
                        color: WHITE.into(),
                        shadows_enabled: true,
                        inner_angle: 0.6,
                        outer_angle: 0.8,
                        ..default()
                    },
                    ..default()
                });
            });

        commands.spawn((
            SpatialBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))),
            SpatialListener::new(4.0),
        ));
    }
}

fn toggle_pause(
    state: Res<State<GameplayState>>,
    action_state: Res<ActionState<PlayerAction>>,
    mut next_state: ResMut<NextState<GameplayState>>,
) {
    if action_state.just_pressed(&PlayerAction::Pause) {
        match state.get() {
            GameplayState::Playing => next_state.set(GameplayState::Paused),
            GameplayState::Paused => next_state.set(GameplayState::Playing),
        }
    }
}

fn control_camera(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let mut player_transform = query.single_mut();
    let move_delta =
        time.delta_seconds() * action_state.clamped_axis_pair(&PlayerAction::Move).xy();
    player_transform.translation += Vec3::new(move_delta.x, 0.0, move_delta.y);

    // impl pause
    if action_state.just_pressed(&PlayerAction::Pause) {
        println!("Paused!")
    }
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

fn shoot(
    action_state: Res<ActionState<PlayerAction>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    query: Query<Entity, With<Tower>>,
) {
    if action_state.just_pressed(&PlayerAction::Shoot) {
        let parent = query.single();
        commands.entity(parent).with_children(|p| {
            p.spawn((
                ShotTimer {
                    timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                },
                Bullet {
                    direction: Vec3::new(0.0, 0.0, 1.0),
                    speed: 5.0,
                },
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.05)),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.45, 0.0)),
                    ..default()
                },
            ));
        });
    }
}

fn move_bullets(mut bullets: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut transform) in &mut bullets {
        transform.translation += bullet.direction.normalize() * bullet.speed * time.delta_seconds();
    }
}

fn despawn_shots(mut commands: Commands, mut q: Query<(Entity, &mut ShotTimer)>, time: Res<Time>) {
    for (entity, mut shot_timer) in q.iter_mut() {
        // timers gotta be ticked, to work
        shot_timer.timer.tick(time.delta());

        // if it finished, despawn the shot
        if shot_timer.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
