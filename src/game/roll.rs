use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, Actionlike, InputControlKind};

use super::Die;

pub struct RollPlugin;

impl Plugin for RollPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<RollAction>::default())
            .init_resource::<ActionState<RollAction>>()
            .insert_resource(RollAction::default_input_map());
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, Resource)]
#[reflect(Resource)]
enum RollAction {
    HighlightLeft,
    HighlightRight,
    Roll,
}

impl Actionlike for RollAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            RollAction::HighlightLeft => InputControlKind::Button,
            RollAction::HighlightRight => InputControlKind::Button,
            RollAction::Roll => InputControlKind::Button,
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
        input_map.insert(Self::Roll, GamepadButton::South);

        // Default kbm input bindings
        input_map.insert(Self::HighlightLeft, KeyCode::KeyQ);
        input_map.insert(Self::HighlightRight, KeyCode::KeyE);
        input_map.insert(Self::Roll, KeyCode::Space);

        input_map
    }
}

#[derive(Resource)]
struct DiePool {
    highlighted: usize,
    items: Vec<Die>,
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
            item.roll();
        }
    }
}
