use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{egui::{self, Color32}, EguiContexts};
use livekit::DataPacketKind;

use crate::{character::{resources::CharacterAnimation, systems::push_character}, editor::{resources::{Scene, StateName}, systems::create_scene, CreateSceneEvent}, main_menu::resources::RoomData, map_structures::{components::{InteractiveEvent, InteractiveType}, resources::MapData}, networking::{rtc::{resources::RTCResource, systems::{create_room, leave_room}}, s3::{resources::S3Data, systems::load_tilemap_from_s3}}, utils::{from_grid_xy_to_index, from_xy_to_grid}, wardrobe::{resources::{OnWardrobeConfirmed, WardrobeResources}, WardrobeState}, FocusState};

use super::{components::{Depth, TriggerButton}, GameResources, MultiplayerRoomState};

pub fn load_game_assets(mut commands: Commands, asset_server: Res<AssetServer>){
    let z_button_handle: Handle<Image> = asset_server.load("icon/z_button.png");
    commands.spawn((
        SpriteBundle {
            texture: z_button_handle,
            visibility: Visibility::Hidden,
            ..default()
        },
        TriggerButton {pos_x: 0.0, pos_y: 0.0, trigger: Vec::new()}
    ));
}

pub fn display_room_ui(mut commands: Commands, mut game_resources: ResMut<GameResources>, mut room_data: ResMut<RoomData>, mut create_scene_event: EventWriter<CreateSceneEvent>, mut contexts: EguiContexts, mut wardrobe_resources: ResMut<WardrobeResources>, mut rtc_resource: ResMut<RTCResource>, mut focus_state: ResMut<NextState<FocusState>>, mut wardrobe_state: ResMut<NextState<WardrobeState>>, mut is_wardrobe_opened: Local<bool>, mut is_change_map_opened: Local<bool>){

    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("Room Settings")
    .frame(egui::Frame{..default()})
    .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-5.0 ,5.0))
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            
            egui::menu::menu_button(ui, "more ...", |ui| {
                if ui.button(
                    match *is_wardrobe_opened {
                        true => "close wardrobe",
                        false => "open wardrobe",
                    }
                ).clicked() {
                    wardrobe_resources.is_name_changable = true;
                    wardrobe_resources.is_clothes_changable = true;
                    wardrobe_resources.is_map_changable = false;
                    wardrobe_resources.display_comfirm_button = false;
                    if *is_wardrobe_opened {
                        wardrobe_state.set(WardrobeState::Close);
                    } else {
                        wardrobe_state.set(WardrobeState::Open);
                    }
                    *is_wardrobe_opened = !*is_wardrobe_opened;
                    *is_change_map_opened = false;
                }
                if ui.button(
                    match *is_change_map_opened {
                        true => "close map",
                        false => "change map",
                    }
                ).clicked() {
                    wardrobe_resources.is_name_changable = false;
                    wardrobe_resources.is_clothes_changable = false;
                    wardrobe_resources.is_map_changable = true;
                    wardrobe_resources.display_comfirm_button = false;
                    if *is_change_map_opened {
                        wardrobe_state.set(WardrobeState::Close);
                    } else {
                        wardrobe_state.set(WardrobeState::Open);
                    }
                    *is_wardrobe_opened = false;
                    *is_change_map_opened = !*is_change_map_opened;
                }
            });

            if ui.button("invite")
                .on_hover_text(room_data.room_id.to_string())
                .clicked() {
                ui.output_mut(|output| {
                    output.copied_text = room_data.room_id.to_string();
                });
            }

            if ui.button("leave").clicked() {
                leave_room(&mut room_data, &mut rtc_resource);

                *is_wardrobe_opened = false;
                *is_change_map_opened = false;
                create_scene(&mut commands, &mut create_scene_event, &mut room_data, "select room", HashMap::from([
                    (StateName::MainMenuState as u8, 2),
                    (StateName::RoomMetadataListener as u8, 1)
                ]), None);
            }
        });
    });

    egui::Window::new("Room Chat")
    .frame(egui::Frame{rounding: egui::Rounding { nw: 5.0, ne: 5.0, sw: 0.0, se: 0.0 }, fill: Color32::from_rgba_premultiplied(0, 0, 0, 200), ..default()})
    .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(5.0 ,-5.0))
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {
        ui.vertical(|ui| {

            for message in game_resources.chat_messages.iter() {
                ui.label(message);
            }

            let chatbar_response = ui.text_edit_singleline(&mut game_resources.chatbar);
            if chatbar_response.gained_focus() {
                focus_state.set(FocusState::UI);
            }
            if chatbar_response.lost_focus() {
                focus_state.set(FocusState::Game);
                if !game_resources.chatbar.is_empty() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut username = String::from("anonymous");
                    if !room_data.this_user_name.is_empty() {
                        username = room_data.this_user_name.to_string();
                    }
                    let message = game_resources.chatbar.to_string();
                    let fmt_message = format!("{}: {}", username, message);

                    game_resources.chat_messages.push(fmt_message.to_string());
                    rtc_resource.send_message(room_data.room_users.len(), &room_data.room_id, &room_data.this_user_uuid, "chat", &fmt_message, DataPacketKind::Reliable);
                    
                    if game_resources.chat_messages.len() > 15 {
                        game_resources.chat_messages.remove(0);
                    }
                    game_resources.chatbar.clear();
                }
            }
        });
    });
}

pub fn setup_room(mut commands: Commands, asset_server: Res<AssetServer>, scene: Res<Scene>, mut map_data: ResMut<MapData>, mut room_data: ResMut<RoomData>, mut s3_data: ResMut<S3Data>, mut character_animation: Res<CharacterAnimation>, mut multiplayer_room_state: ResMut<NextState<MultiplayerRoomState>>){

    map_data.is_map_loaded = false;
    load_tilemap_from_s3(&mut commands, &scene.scene_uuid, &mut s3_data, "shalloville".to_string(), map_data.map_name.to_string());

    room_data.load_ready_users(&mut commands, &asset_server, &scene.scene_uuid, &mut s3_data, &mut map_data, &mut character_animation);

    multiplayer_room_state.set(MultiplayerRoomState::Consumed);
}

pub fn update_depth(mut depth_query: Query<(&Depth, &mut Transform), Changed<Depth>>){
    for (depth, mut transform) in depth_query.iter_mut() {
        transform.translation.z = depth.index;
    }
}

pub fn display_trigger_button(mut trigger_button_query: Query<(&TriggerButton, &mut Transform, &mut Visibility), Changed<TriggerButton>>, map_data: Res<MapData>, mut game_resources: ResMut<GameResources>){
    if let Ok((z_button, mut transform, mut visibility)) = trigger_button_query.get_single_mut() {
        *visibility = Visibility::Visible;
        transform.translation = Vec3::new(z_button.pos_x, z_button.pos_y, map_data.max_y + 10.0);
        game_resources.right_bottom_texts.clear();
        for (keycode, interactive_type) in z_button.trigger.iter() {
            let mut message = match keycode {
                KeyCode::KeyZ => String::from("[Z]"),
                KeyCode::KeyX => String::from("[X]"),
                KeyCode::KeyC => String::from("[C]"),
                _ => String::from("[undefined]")
            };
            message.push(' ');
            message.push_str(match interactive_type {
                InteractiveType::SwitchScreenShare => "On/Off Screen Share",
                InteractiveType::SwitchCameraShare => "On/Off Camera Share",
                InteractiveType::None => ""
            });
            game_resources.right_bottom_texts.push(message);
        }
    } 
}

pub fn hide_trigger_button(mut trigger_button_query: Query<&mut Visibility, With<TriggerButton>>) {
    if let Ok(mut visibility) = trigger_button_query.get_single_mut() {
        *visibility = Visibility::Hidden;
    }
}

pub fn trigger_button_onpress(keys: Res<ButtonInput<KeyCode>>, trigger_button_query: Query<&TriggerButton>, mut interactive_event: EventWriter<InteractiveEvent>, map_data: Res<MapData>){
    if let Ok(trigger_button) = trigger_button_query.get_single() {
        for (keycode, interactive_type) in trigger_button.trigger.iter() {
            if keys.just_pressed(*keycode) {
                let (grid_x, grid_y) = from_xy_to_grid(trigger_button.pos_x, trigger_button.pos_y, map_data.tile_width, map_data.tile_height);
                let index = from_grid_xy_to_index(grid_x, grid_y, map_data.map_col as f32);
                interactive_event.send(InteractiveEvent((interactive_type.clone(), index)));
            }
        }
    }
}

pub fn right_bottom_window(mut contexts: EguiContexts, mut game_resources: ResMut<GameResources>){
    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("Right Bottom")
    .frame(egui::Frame{..default()})
    .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::ZERO)
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {

        for message in game_resources.right_bottom_texts.iter_mut() {
            ui.add_enabled(false, egui::TextEdit::singleline(message));
        }
    });
}