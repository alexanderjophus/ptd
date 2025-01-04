use bevy::prelude::*;

use crate::GameState;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Economy { money: 100 })
            .add_systems(OnEnter(GameState::Game), economy_setup)
            .add_systems(Update, (update_resources).run_if(in_state(GameState::Game)));
    }
}

#[derive(Resource)]
pub struct Economy {
    pub money: u32,
}

#[derive(Component)]
pub struct ResourcesTextOverlay;

fn economy_setup(mut commands: Commands, economy: Res<Economy>) {
    commands.spawn((
        Text(format!("Money: {}", economy.money)),
        ResourcesTextOverlay,
    ));
}

fn update_resources(mut query: Query<(&mut Text, &ResourcesTextOverlay)>, economy: Res<Economy>) {
    for (mut text, _) in query.iter_mut() {
        text.0 = format!("Money: {}", economy.money);
    }
}
