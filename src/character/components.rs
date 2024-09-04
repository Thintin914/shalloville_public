use std::collections::HashMap;
use bevy::prelude::*;

#[derive(Component)]
pub struct Character {
    pub uuid: String,
    pub current_frame: i8,
    pub increment: i8,

    pub entity_parts: HashMap<String, (Entity, Vec3)>,
}