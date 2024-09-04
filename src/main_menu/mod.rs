use bevy::prelude::*;

mod systems;
use num_enum::TryFromPrimitive;
use systems::*;

pub mod resources;
use resources::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<RoomData>()
        .init_state::<MainMenuState>()
        .add_systems(OnEnter(MainMenuState::InitInputRoom), init_select_room)
        .add_systems(OnEnter(MainMenuState::InputRoom), setup_select_room)
        .add_systems(Update, select_room.run_if(in_state(MainMenuState::InputRoom)))
        .add_systems(OnEnter(MainMenuState::InputUserSkin), setup_select_user_skin)
        .add_systems(Update, select_user_skin.run_if(in_state(MainMenuState::InputUserSkin)));
    }
}

#[repr(u8)]
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, TryFromPrimitive)]
pub enum MainMenuState {
    None = 0,
    #[default]
    InitInputRoom = 1,
    InputRoom = 2,
    InputUserSkin = 3
}