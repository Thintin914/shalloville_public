use std::collections::HashMap;
use uuid::Uuid;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{character::{resources::CharacterAnimation, systems::push_character}, editor::{resources::{Scene, StateName}, systems::create_scene, CreateSceneEvent}, map_structures::resources::MapData, networking::{rtc::{resources::RTCResource, systems::check_room}, s3::{resources::S3Data, systems::load_tilemap_from_s3}}, wardrobe::resources::{OnWardrobeConfirmed, WardrobeResources}};

use super::{ MainMenuImages, RoomData, UserData, UserStatus};

pub fn init_select_room(mut commands: Commands, mut room_data: ResMut<RoomData>, mut create_scene_event: EventWriter<CreateSceneEvent>) {
    create_scene(&mut commands, &mut create_scene_event, &mut room_data, "lobby", HashMap::from([
        (StateName::MainMenuState as u8, 2),
        (StateName::RoomMetadataListener as u8, 1)
    ]), None);
}

pub fn setup_select_room(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>, room_data: ResMut<RoomData>, images: Local<MainMenuImages>) {
    
    let window = window_query.get_single().unwrap();

    let bg_handle: Handle<Image> = images.background.clone_weak();
    let bg =
    commands.spawn(
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            texture: bg_handle,
            ..default()
        }
    ).id();

    let canvas_handle: Handle<Image> = images.canvas.clone_weak();
    let canvas =
    commands.spawn(
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            texture: canvas_handle,
            ..default()
        }
    ).id();

    let icon_handle: Handle<Image> = images.icon.clone_weak();
    let icon =
    commands.spawn(
        SpriteBundle {
            transform: Transform::from_xyz(0.0, window.height() * 0.35, 2.0),
            texture: icon_handle,
            ..default()
        }
    ).id();

    commands.entity(room_data.current_scene.unwrap()).add_child(bg).add_child(canvas).add_child(icon);
}

pub fn select_room(mut commands: Commands, mut contexts: EguiContexts, scene: Res<Scene>, window_query: Query<&Window, With<PrimaryWindow>>, mut create_scene_event: EventWriter<CreateSceneEvent>, mut room_data: ResMut<RoomData>, mut rtc_resource: ResMut<RTCResource>, mut wardrobe_resources: ResMut<WardrobeResources>){
    let window = window_query.get_single().unwrap();
    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("Select Room")
    .frame(egui::Frame{..default()})
    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0 ,0.0))
    .min_width(window.width() * 0.3)
    .max_width(500.0)
    .max_height(100.0)
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {

        ui.vertical_centered(|ui| {
            if ui.button("New Room").clicked() {

                wardrobe_resources.is_name_changable = true;
                wardrobe_resources.is_clothes_changable = true;
                wardrobe_resources.is_map_changable = true;
                wardrobe_resources.display_comfirm_button = true;
                wardrobe_resources.on_confirm_text = "Create Room".to_string();
                wardrobe_resources.on_confirm_action = OnWardrobeConfirmed::CreateRoom;

                create_scene(&mut commands, &mut create_scene_event, &mut room_data, "wardrobe", HashMap::from([
                    (StateName::MainMenuState as u8, 3),
                    (StateName::WardrobeState as u8, 1),
                    (StateName::TriggerButtonState as u8, 1),
                    (StateName::MovementState as u8, 1),
                    (StateName::CharacterExistState as u8, 1),
                    (StateName::CameraState as u8, 2),
                    (StateName::RoomMetadataListener as u8, 1)
                ]), Some(Vec::from([
                    StateName::StreamingState as u8
                ])));
            }

            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                ui.text_edit_singleline(&mut room_data.room_id);
                if ui.button("Enter").clicked() {
                    check_room(&mut commands, &scene.scene_uuid, &mut rtc_resource, room_data.room_id.to_string());
                }
            });
        });
    });
}

pub fn setup_select_user_skin(mut commands: Commands, assets_server: Res<AssetServer>, scene: Res<Scene>, mut character_animation: Res<CharacterAnimation>, mut room_data: ResMut<RoomData>, wardrobe_resources: Res<WardrobeResources>, mut map_data: ResMut<MapData>, mut s3_data: ResMut<S3Data>) {

    room_data.room_users.clear();

    map_data.is_map_loaded = false;
    load_tilemap_from_s3(&mut commands, &scene.scene_uuid, &mut s3_data, "shalloville".to_string(), "0".to_string());

    let uuid = Uuid::new_v4().to_string();

    room_data.this_user_uuid = uuid.to_string();
    let mut user_data = UserData::create_empty(uuid.as_str(), wardrobe_resources.wardrobe_parts.body_parts.clone());
    push_character(&mut commands, &assets_server, &scene.scene_uuid, &mut s3_data, &mut map_data, &mut room_data, &mut character_animation, &mut user_data);
}

pub fn select_user_skin(mut commands: Commands, mut contexts: EguiContexts, mut room_data: ResMut<RoomData>, mut create_scene_event: EventWriter<CreateSceneEvent>){
    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("Select User Skin - Back")
    .frame(egui::Frame{..default()})
    .anchor(egui::Align2::LEFT_TOP, egui::vec2(5.0 ,5.0))
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {
        if ui.button("Back").clicked() {
            create_scene(&mut commands, &mut create_scene_event, &mut room_data, "lobby", HashMap::from([
                (StateName::MainMenuState as u8, 2),
                (StateName::RoomMetadataListener as u8, 1)
            ]), None);
        }
    }); 
}