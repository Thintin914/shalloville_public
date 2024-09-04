use bevy::prelude::*;
use num_enum::TryFromPrimitive;

mod components;

pub mod resources;
use resources::*;

mod systems;
use systems::*;

pub struct NokhwaPlugin;

impl Plugin for NokhwaPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<StreamingState>()
        .init_resource::<NokhwaCamera>()
        .init_resource::<StreamingResources>()
        .add_systems(Update, detect_device_camera)
        .add_systems(OnEnter(StreamingState::Close), close_device_camera)
        .add_systems(Update, update_device_camera_image.run_if(in_state(StreamingState::Open)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum StreamingState {
    Open = 0,
    #[default]
    Close = 1
}