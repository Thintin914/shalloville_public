use bevy::prelude::*;

pub mod systems;
use systems::*;

pub mod resources;
use resources::*;

pub mod components;

pub struct S3Plugin;

impl Plugin for S3Plugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<S3Data>()
        .add_systems(Update, execute_tasks);
    }
}