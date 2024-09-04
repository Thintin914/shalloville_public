use std::collections::HashMap;

use bevy::prelude::*;
use num_enum::TryFromPrimitive;

use crate::{camera::CameraState, character::{CharacterExistState, MovementState}, game::{MultiplayerRoomState, TriggerButtonState}, main_menu::MainMenuState, networking::rtc::RoomMetadataListener, nokhwa::StreamingState, wardrobe::WardrobeState};

use super::EditorState;

#[derive(Resource)]
pub struct DefaultStates {
    pub hashmap: HashMap<u8, u8>,
    pub prev: Vec<u8>,
    pub dynamic: Vec<u8>,
}

#[derive(Resource)]
pub struct Scene {
    pub scene_name: String,
    pub scene_uuid: String
}

impl Default for Scene {
    fn default() -> Scene {
        Scene {
            scene_name: String::new(),
            scene_uuid: String::new()
        }
    }
}

impl Default for DefaultStates {
    fn default() -> DefaultStates {
        DefaultStates {
            hashmap: HashMap::from([
                (StateName::MainMenuState as u8, MainMenuState::None as u8),
                (StateName::WardrobeState as u8, WardrobeState::Close as u8),
                (StateName::StreamingState as u8, StreamingState::Close as u8),
                (StateName::TriggerButtonState as u8, TriggerButtonState::Hidden as u8),
                (StateName::CameraState as u8, CameraState::Static as u8),
                (StateName::MovementState as u8, MovementState::Unmovable as u8),
                (StateName::CharacterExistState as u8, CharacterExistState::NotExist as u8),
                (StateName::MultiplayerRoomState as u8, MultiplayerRoomState::False as u8),
                (StateName::EditorState as u8, EditorState::Close as u8),
                (StateName::RoomMetadataListener as u8, RoomMetadataListener::Close as u8)

            ]),
            prev: Vec::new(),
            dynamic: Vec::new()
        }
    }
}

#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum StateName {
    MainMenuState,
    WardrobeState,
    StreamingState,
    TriggerButtonState,
    CameraState,
    MovementState,
    CharacterExistState,
    MultiplayerRoomState,
    EditorState,
    RoomMetadataListener
}