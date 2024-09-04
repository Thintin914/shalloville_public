use bevy::prelude::*;

use bevy::{
    ecs::world::CommandQueue,
    tasks::Task,
};

#[derive(Component)]
pub struct ComputeTask(pub (String, Task<CommandQueue>));

#[derive(Component)]
pub struct LoadTilemapTask {
    pub map_name: String,
    pub map_str: String
}

#[derive(Component)]
pub struct LoadSpriteTask {
    pub image_handle: Handle<Image>,
    pub entity: Entity
}