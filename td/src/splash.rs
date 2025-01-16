use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::game::{AllAssets, AssetCollections, GltfAssets};

use super::{despawn_screen, GameState, GAME_NAME};

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Splash)
                .continue_to_state(GameState::Menu)
                .load_collection::<GltfAssets>()
                .load_collection::<AllAssets>()
                .register_dynamic_asset_collection::<AssetCollections>()
                .with_dynamic_assets_file::<AssetCollections>("game.ron"),
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
    commands.spawn((Camera2d, SplashCamera));

    commands.spawn((Text(GAME_NAME.to_string()), OnSplashScreen));
}
