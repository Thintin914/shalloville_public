mod build;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod components;

mod networking;
use build::init;
use networking::NetworkingPlugin;

mod camera;
use camera::CameraPlugin;

mod mouse;
use mouse::*;

mod map_structures;
use map_structures::*;

mod utils;

mod editor;
use editor::EditorPlugin;

mod main_menu;
use main_menu::MainMenuPlugin;

mod wardrobe;
use wardrobe::WardrobePlugin;

mod character;
use character::CharacterPlugin;

mod game;
use game::GamePlugin;

mod nokhwa;
use nokhwa::NokhwaPlugin;

fn main() {
    init();

    App::new()
    .add_plugins(DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shalloville".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        })
    )
    .add_plugins(EguiPlugin)
    .add_plugins(NetworkingPlugin)
    // .add_plugins(WorldInspectorPlugin::new())
    .add_plugins(EditorPlugin)
    .add_plugins(CameraPlugin)
    .add_plugins(MousePlugin)
    .add_plugins(MapStructuresPlugin)
    .add_plugins(MainMenuPlugin)
    .add_plugins(WardrobePlugin)
    .add_plugins(CharacterPlugin)
    .add_plugins(GamePlugin)
    .add_plugins(NokhwaPlugin)
    .run();
}

#[derive(Default)]
pub struct AnimationTimer {
    pub collasped_timer: f32
}