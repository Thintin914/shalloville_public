use bevy::prelude::*;

pub mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod components;

pub mod resources;
use resources::*;

use crate::FocusState;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<CharacterAnimation>()
        .init_state::<MovementState>()
        .init_state::<CharacterExistState>()
        .add_systems(Update, update_character_animation.run_if(in_state(CharacterExistState::Exist)))
        .add_systems(Update, move_character_on_input.run_if(in_state(MovementState::Movable).and_then(in_state(FocusState::Game))))
        .add_systems(Update, update_character_controller.run_if(in_state(CharacterExistState::Exist)))
        .add_systems(Update, on_tilemap_loaded.run_if(in_state(CharacterExistState::Exist)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum MovementState {
    #[default]
    Unmovable = 0,
    Movable = 1
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum CharacterExistState {
    #[default]
    NotExist = 0,
    Exist = 1
}