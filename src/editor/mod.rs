use std::collections::HashMap;
use num_enum::TryFromPrimitive;

use bevy::prelude::*;
use bevy_entitiles::EntiTilesPlugin;

mod components;

pub mod resources;
use resources::*;

pub mod systems;
use systems::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<DefaultStates>()
        .init_resource::<Scene>()
        .init_state::<EditorState>()
        .add_plugins(EntiTilesPlugin)
        .add_event::<TilemapLoadedEvent>()
        .add_event::<CreateSceneEvent>()
        .add_systems(Startup, setup_tilemap_editor.run_if(in_state(EditorState::Open)))
        .add_systems(Update, tilemap_editor.run_if(in_state(EditorState::Open)))
        .add_systems(Update, update_preview_image.run_if(in_state(EditorState::Open)))
        .add_systems(Update, load_tilemap_event_listener)
        .add_systems(Update, load_sprite_event_listener)
        .add_systems(Update, on_create_scene);
    }
}

#[derive(Event)]
pub struct TilemapLoadedEvent();

#[derive(Event)]
pub struct CreateSceneEvent(pub (String, HashMap<u8, u8>, Vec<u8>));

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum EditorState {
    #[default]
    Close = 0,
    Open = 1
}