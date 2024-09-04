use bevy::prelude::*;

mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod resources;
use resources::*;

use crate::{editor::EditorState, TilemapEditorState};

mod components;

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<MouseData>()
        .init_state::<FocusState>()
        .add_systems(Startup, init_mouse)
        .add_systems(Update, update_window_size)
        .add_systems(Update, update_mouse_pointer)
        .add_systems(Update, onclick_place_tile.run_if(in_state(TilemapEditorState::PlaceTile).and_then(in_state(FocusState::Game))))
        .add_systems(Update, onclick_place_entity.run_if(in_state(TilemapEditorState::PlaceEntity).and_then(in_state(FocusState::Game))))
        .add_systems(Update, onclick_place_animated_entity.run_if(in_state(TilemapEditorState::PlaceAnimatedEntity).and_then(in_state(FocusState::Game))))
        .add_systems(Update, onclick_place_hitbox.run_if(in_state(TilemapEditorState::PlaceHitbox).and_then(in_state(FocusState::Game))))
        .add_systems(Update, onclick_place_info.run_if(in_state(TilemapEditorState::PlaceInfo).and_then(in_state(FocusState::Game))))
        .add_systems(Update, onpress_remove_entity.run_if(in_state(EditorState::Open)));

    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum FocusState {
    UI = 0,
    #[default]
    Game = 1
}