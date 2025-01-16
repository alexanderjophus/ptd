#[cfg(feature = "debug")]
mod debug;
mod game;
mod input;
mod menu;
mod splash;

use bevy::{asset::AssetMetaCheck, prelude::*};

pub const GAME_NAME: &str = "Jophus' TD";

// Enum that will be used as a global state for the game
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        // Declare the game state
        .init_state::<GameState>()
        // Adds the plugins for each state
        .add_plugins((
            splash::SplashPlugin,
            menu::MenuPlugin,
            game::GamePlugin,
            input::InputModeManagerPlugin,
            #[cfg(feature = "debug")]
            debug::DebugPlugin,
        ))
        .run();
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
