use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::game::{AssetCollections, EnemyAssets, GltfAssets, Resources, TowerAssets};

use super::{despawn_screen, GameState, GAME_NAME};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        // As this plugin is managing the splash screen, it will focus on the state `GameState::Splash`
        app.add_loading_state(
            LoadingState::new(GameState::Splash)
                .continue_to_state(GameState::Menu)
                .load_collection::<TowerAssets>()
                .load_collection::<EnemyAssets>()
                .load_collection::<GltfAssets>()
                .register_dynamic_asset_collection::<AssetCollections>()
                .with_dynamic_assets_file::<AssetCollections>("towers.game.ron")
                .with_dynamic_assets_file::<AssetCollections>("enemies.game.ron")
                .init_resource::<Resources>(),
        )
        // When entering the state, spawn everything needed for this screen
        .add_systems(OnEnter(GameState::Splash), splash_setup)
        // When exiting the state, despawn everything that was spawned for this screen
        .add_systems(
            OnExit(GameState::Splash),
            (
                despawn_screen::<OnSplashScreen>,
                despawn_screen::<SplashCamera>,
            ),
        );
    }
}

// Tag component used to tag entities added on the splash screen
#[derive(Component)]
struct OnSplashScreen;

#[derive(Component)]
struct SplashCamera;

fn splash_setup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), SplashCamera));

    commands.spawn((Text(GAME_NAME.to_string()), OnSplashScreen));
}
