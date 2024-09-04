use bevy::prelude::*;
use num_enum::TryFromPrimitive;

pub mod components;

pub mod systems;
use systems::*;

pub mod resources;
use resources::*;

pub mod video;
pub mod video_renderer;

use crate::game::MultiplayerRoomState;

pub struct RTCPlugin;

impl Plugin for RTCPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<RTCResource>()
        .init_state::<RoomMetadataListener>()
        .add_systems(Update, load_room_metadata_event_listener.run_if(in_state(RoomMetadataListener::Open)))
        .add_systems(Update, on_room_event_received.run_if(in_state(MultiplayerRoomState::Consumed)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum RoomMetadataListener {
    #[default]
    Close = 0,
    Open = 1
}