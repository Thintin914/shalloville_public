use bevy::prelude::*;

use crate::wardrobe::resources::BodyParts;

#[derive(Component)]
pub struct LoadMetadataTask {
    pub metadata: String
}

#[derive(Component)]
pub struct MultiplayerUserAttribute {
    pub username: String,
    pub body_part: BodyParts
}