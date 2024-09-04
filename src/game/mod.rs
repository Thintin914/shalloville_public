use bevy::prelude::*;

pub mod components;

pub mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod resources;
use resources::*;

use crate::editor::EditorState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<GameResources>()
        .init_state::<MultiplayerRoomState>()
        .init_state::<TriggerButtonState>()
        .add_systems(Startup, load_game_assets.run_if(in_state(EditorState::Close)))
        .add_systems(Update, update_depth)
        .add_systems( Update, display_room_ui.run_if(in_state(MultiplayerRoomState::Consumed)))
        .add_systems(OnEnter(MultiplayerRoomState::True), setup_room)
        .add_systems(Update, display_trigger_button.run_if(in_state(TriggerButtonState::Display)))
        .add_systems(OnEnter(TriggerButtonState::Hidden), hide_trigger_button)
        .add_systems(Update, trigger_button_onpress.run_if(in_state(TriggerButtonState::Display)))
        .add_systems(Update, right_bottom_window.run_if(in_state(TriggerButtonState::Display)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum MultiplayerRoomState {
    #[default]
    False = 0,
    True = 1,
    Consumed = 2
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum TriggerButtonState {
    #[default]
    Hidden = 0,
    Display = 1
}