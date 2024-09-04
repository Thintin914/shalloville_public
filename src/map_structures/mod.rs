use bevy::prelude::*;

pub mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod resources;
use resources::*;

pub mod components;
use components::*;

pub struct MapStructuresPlugin;

impl Plugin for MapStructuresPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<MapData>()
        .init_resource::<MapStructures>()
        .init_state::<TilemapEditorState>()
        .add_event::<InteractiveEvent>()
        .add_systems(Startup, load_spritesheet)
        .add_systems(Update, update_animated_sprite);
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum TilemapEditorState {
    #[default]
    Free = 0,
    PlaceTile = 1,
    PlaceEntity = 2,
    PlaceAnimatedEntity = 3,
    PlaceHitbox = 4,
    PlaceInfo = 5
}