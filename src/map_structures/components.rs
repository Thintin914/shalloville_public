use bevy::prelude::*;

#[derive(Component)]
pub struct MapStructure;

#[derive(Component)]
pub struct SpriteTag {
    pub index: i8,
    pub pos: i32
}

#[derive(Component)]
pub struct AnimatedTag {
    pub index: i8,
    pub pos: i32,
    pub current: i8,
    pub last: i8,
    pub increment: i8
}

#[derive(Component, Clone, PartialEq, Eq)]
pub struct InteractiveTrigger {
    pub trigger: Vec<(KeyCode, InteractiveType)>
}

#[derive(Clone, PartialEq, Eq)]
pub enum InteractiveType {
    None,
    SwitchScreenShare,
    SwitchCameraShare,
}

#[derive(Event)]
pub struct InteractiveEvent(pub (InteractiveType, i32));