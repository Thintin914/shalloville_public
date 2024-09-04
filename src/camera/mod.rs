use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use bevy::prelude::*;

mod systems;
use systems::*;

pub mod components;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<CameraState>()
        .add_systems(Startup, create_camera)
        .add_systems(OnEnter(CameraState::Static), camera_static)
        .add_systems(Update, move_camera_free.run_if(in_state(CameraState::Free)))
        .add_systems(Update, camera_follow_user.run_if(in_state(CameraState::Follow)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum CameraState {
    #[default]
    Static = 0,
    Free = 1,
    Follow = 2,
    Locked = 3,
}