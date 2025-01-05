use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, Actionlike, InputControlKind};

use crate::GameState;

use super::GamePlayState;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<EconomyAction>::default())
            .init_resource::<ActionState<EconomyAction>>()
            .insert_resource(EconomyAction::default_input_map())
            .insert_resource(Economy { money: 100 })
            .add_systems(OnEnter(GameState::Game), economy_setup)
            .add_systems(
                Update,
                (buy_die, start_placement)
                    .run_if(in_state(GamePlayState::Economy).and(in_state(GameState::Game))),
            )
            .add_systems(Update, update_resources.run_if(in_state(GameState::Game)));
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, Resource)]
#[reflect(Resource)]
enum EconomyAction {
    ToggleDieLeft,
    ToggleDieRight,
    BuyDie,
    PlacementPhase,
}

impl Actionlike for EconomyAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            EconomyAction::ToggleDieLeft => InputControlKind::Button,
            EconomyAction::ToggleDieRight => InputControlKind::Button,
            EconomyAction::BuyDie => InputControlKind::Button,
            EconomyAction::PlacementPhase => InputControlKind::Button,
        }
    }
}

impl EconomyAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert(Self::ToggleDieLeft, GamepadButton::DPadLeft);
        input_map.insert(Self::ToggleDieRight, GamepadButton::DPadRight);
        input_map.insert(Self::BuyDie, GamepadButton::East);
        input_map.insert(Self::PlacementPhase, GamepadButton::South);

        // // Default kbm input bindings
        input_map.insert(Self::ToggleDieLeft, KeyCode::KeyQ);
        input_map.insert(Self::ToggleDieRight, KeyCode::KeyE);
        input_map.insert(Self::BuyDie, KeyCode::Space);
        input_map.insert(Self::PlacementPhase, KeyCode::Enter);

        input_map
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

fn buy_die(action_state: Res<ActionState<EconomyAction>>, mut economy: ResMut<Economy>) {
    if action_state.just_pressed(&EconomyAction::BuyDie) {
        economy.money -= 10;
    }
}

fn start_placement(
    action_state: Res<ActionState<EconomyAction>>,
    mut next_state: ResMut<NextState<GamePlayState>>,
) {
    if action_state.just_pressed(&EconomyAction::PlacementPhase) {
        next_state.set(GamePlayState::Placement);
    }
}

fn update_resources(mut query: Query<(&mut Text, &ResourcesTextOverlay)>, economy: Res<Economy>) {
    for (mut text, _) in query.iter_mut() {
        text.0 = format!("Money: {}", economy.money);
    }
}
