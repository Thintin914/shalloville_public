use bevy::prelude::*;

#[derive(Component)]
pub struct EditorPreviewImage {
    pub is_updated: bool,
    pub image: Option<Entity>
}

