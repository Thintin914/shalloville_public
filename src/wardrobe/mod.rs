use bevy::prelude::*;

pub mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod resources;
use resources::*;

pub struct WardrobePlugin;

impl Plugin for WardrobePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<WardrobeResources>()
        .init_state::<WardrobeState>()
        .add_systems(Update, wardrobe_editor.run_if(in_state(WardrobeState::Open)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum WardrobeState {
    #[default]
    Close = 0,
    Open = 1
}