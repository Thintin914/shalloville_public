use bevy::prelude::*;

use crate::map_structures::components::InteractiveType;

#[derive(Component)]
pub struct Depth {
    pub index: f32
}

#[derive(Component)]
pub struct TriggerButton {
    pub pos_x: f32,
    pub pos_y: f32,
    pub trigger: Vec<(KeyCode, InteractiveType)>
}