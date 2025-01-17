use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, Actionlike, InputControlKind};

use crate::{despawn_screen, GameState};

use super::{BaseElementType, Die, DieBuilder, DiePurchaseEvent, GamePlayState};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<EconomyAction>::default())
            .init_resource::<ActionState<EconomyAction>>()
            .insert_resource(EconomyAction::default_input_map())
            .insert_resource(Economy { money: 50 })
            .insert_resource(DieShop {
                highlighted: 0,
                items: vec![
                    DieBuilder::from_type(BaseElementType::Fire).build(),
                    DieBuilder::from_type(BaseElementType::Water).build(),
                    DieBuilder::from_type(BaseElementType::Earth).build(),
                    DieBuilder::from_type(BaseElementType::Wind).build(),
                ],
            })
            .add_systems(OnEnter(GameState::Game), economy_setup)
            .add_systems(
                Update,
                (choose_die, display_shop, start_rolling)
                    .run_if(in_state(GamePlayState::Economy).and(in_state(GameState::Game))),
            )
            .add_systems(
                OnExit(GamePlayState::Economy),
                despawn_screen::<DieShopOverlay>,
            );
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

        // Default kbm input bindings
        input_map.insert(Self::ToggleDieLeft, KeyCode::KeyQ);
        input_map.insert(Self::ToggleDieRight, KeyCode::KeyE);
        input_map.insert(Self::BuyDie, KeyCode::Space);
        input_map.insert(Self::PlacementPhase, KeyCode::Enter);

        input_map
    }
}

#[derive(Resource)]
pub struct Economy {
    pub money: usize,
}

#[derive(Resource, Debug, Clone, PartialEq)]
struct DieShop {
    items: Vec<Die>,
    highlighted: usize,
}

#[derive(Component)]
pub struct DieShopOverlay;

fn economy_setup(mut commands: Commands) {
    commands.spawn((Text::default(), DieShopOverlay));
}

fn choose_die(
    action_state: Res<ActionState<EconomyAction>>,
    mut economy: ResMut<Economy>,
    mut shop: ResMut<DieShop>,
    mut ev_die_purchase: EventWriter<DiePurchaseEvent>,
) {
    if action_state.just_pressed(&EconomyAction::ToggleDieLeft) {
        shop.highlighted = (shop.highlighted + shop.items.len() - 1) % shop.items.len();
    }
    if action_state.just_pressed(&EconomyAction::ToggleDieRight) {
        shop.highlighted = (shop.highlighted + 1) % shop.items.len();
    }
    // Buy the die, remove costs, add to diepool resource
    if action_state.just_pressed(&EconomyAction::BuyDie) {
        let cost = shop.items[shop.highlighted].value;
        if economy.money < cost {
            return;
        }
        economy.money -= cost;
        ev_die_purchase.send(DiePurchaseEvent(shop.items[shop.highlighted].clone()));
    }
}

fn display_shop(
    shop: Res<DieShop>,
    economy: Res<Economy>,
    mut query: Query<(&mut Text, &DieShopOverlay)>,
) {
    for (mut text, _) in query.iter_mut() {
        text.0 = format!(
            "Shop\nMoney: {}\n\n{}",
            economy.money,
            shop.items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let prefix = if i == shop.highlighted { ">> " } else { "   " };
                    format!("{}{}", prefix, item)
                })
                .collect::<Vec<String>>()
                .join("\n\n")
        );
    }
}

fn start_rolling(
    action_state: Res<ActionState<EconomyAction>>,
    mut next_state: ResMut<NextState<GamePlayState>>,
) {
    if action_state.just_pressed(&EconomyAction::PlacementPhase) {
        next_state.set(GamePlayState::Rolling);
    }
}
