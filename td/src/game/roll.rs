use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, Actionlike, InputControlKind};

use crate::{despawn_screen, GameState};

use super::{DiePool, DieRolledEvent, GamePlayState};

pub struct RollPlugin;

impl Plugin for RollPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<RollAction>::default())
            .init_resource::<ActionState<RollAction>>()
            .insert_resource(RollAction::default_input_map())
            .add_systems(OnEnter(GamePlayState::Rolling), rolling_setup)
            .add_systems(
                Update,
                (handle_input, display_die_pool)
                    .run_if(in_state(GameState::Game).and(in_state(GamePlayState::Rolling))),
            )
            .add_systems(
                OnExit(GamePlayState::Rolling),
                despawn_screen::<DieRollingOverlay>,
            );
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, Resource)]
#[reflect(Resource)]
enum RollAction {
    HighlightLeft,
    HighlightRight,
    Roll,
    Placement,
}

impl Actionlike for RollAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            RollAction::HighlightLeft => InputControlKind::Button,
            RollAction::HighlightRight => InputControlKind::Button,
            RollAction::Roll => InputControlKind::Button,
            RollAction::Placement => InputControlKind::Button,
        }
    }
}

impl RollAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert(Self::HighlightLeft, GamepadButton::DPadLeft);
        input_map.insert(Self::HighlightRight, GamepadButton::DPadRight);
        input_map.insert(Self::Roll, GamepadButton::East);
        input_map.insert(Self::Placement, GamepadButton::South);

        // Default kbm input bindings
        input_map.insert(Self::HighlightLeft, KeyCode::KeyQ);
        input_map.insert(Self::HighlightRight, KeyCode::KeyE);
        input_map.insert(Self::Roll, KeyCode::Space);
        input_map.insert(Self::Placement, KeyCode::Enter);

        input_map
    }
}

#[derive(Component)]
struct DieRollingOverlay;

fn rolling_setup(mut commands: Commands) {
    // Root node
    commands.spawn((Text::default(), DieRollingOverlay));
}

fn handle_input(
    action_state: Res<ActionState<RollAction>>,
    mut die_pool: ResMut<DiePool>,
    mut next_state: ResMut<NextState<GamePlayState>>,
    mut ev_rolled: EventWriter<DieRolledEvent>,
) {
    if action_state.just_pressed(&RollAction::HighlightLeft) {
        die_pool.highlighted =
            (die_pool.highlighted + die_pool.dice.len() - 1) % die_pool.dice.len();
    }

    if action_state.just_pressed(&RollAction::HighlightRight) {
        die_pool.highlighted = (die_pool.highlighted + 1) % die_pool.dice.len();
    }

    if action_state.just_pressed(&RollAction::Roll) {
        let face = die_pool.roll();
        ev_rolled.send(DieRolledEvent(face));
    }

    if action_state.just_pressed(&RollAction::Placement) {
        next_state.set(GamePlayState::Placement);
    }
}

fn display_die_pool(die_pool: Res<DiePool>, mut query: Query<(&mut Text, &DieRollingOverlay)>) {
    for (mut text, _) in query.iter_mut() {
        text.0 = format!(
            "Die Pool\n\n{}",
            die_pool
                .dice
                .iter()
                .enumerate()
                .map(|(i, die)| {
                    if i == die_pool.highlighted {
                        format!("> {}\n", die)
                    } else {
                        format!("  {}\n", die)
                    }
                })
                .collect::<String>()
        );
    }
}
