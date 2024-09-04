use std::collections::HashMap;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{character::components::Character, editor::{resources::{Scene, StateName}, systems::create_scene, CreateSceneEvent}, game::resources::GameResources, main_menu::resources::RoomData, map_structures::resources::MapData, networking::{rtc::{components::MultiplayerUserAttribute, resources::RTCResource, systems::{create_room, join_room}}, s3::{resources::S3Data, systems::{load_sprite_from_s3, load_tilemap_from_s3}}}, nokhwa::StreamingState, FocusState};

use super::{OnWardrobeConfirmed, WardrobeResources, WardrobeState};

pub fn wardrobe_editor(mut commands: Commands, mut contexts: EguiContexts, scene: Res<Scene>, mut focus_state: ResMut<NextState<FocusState>>, mut wardrobe_state: ResMut<NextState<WardrobeState>>, mut wardrobe_resources: ResMut<WardrobeResources>, mut s3_data: ResMut<S3Data>, window_query: Query<&Window, With<PrimaryWindow>>, mut room_data: ResMut<RoomData>, mut map_data: ResMut<MapData>, mut game_resource: ResMut<GameResources>, mut rtc_resource: ResMut<RTCResource>, mut create_scene_event: EventWriter<CreateSceneEvent>, mut characters_query: Query<&mut Character>){
    let window = window_query.get_single().unwrap();

    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("Wardrobe")
    .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(0.0 ,-window.height() * 0.15))
    .max_width(window.width())
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            if wardrobe_resources.is_name_changable {
                ui.label(egui::RichText::new("Display Name (Optional)").strong().color(egui::Color32::WHITE));
                let username_response = ui.text_edit_singleline(&mut room_data.this_user_name);
    
                if username_response.gained_focus() {
                    focus_state.set(FocusState::UI);
                }
                if username_response.lost_focus() {
                    focus_state.set(FocusState::Game);
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let uuid = room_data.this_user_uuid.to_string();
                        let username = room_data.this_user_name.to_string();
                        if let Some(user_data) = room_data.room_users.get_mut(&uuid) {
                            user_data.username = username;
                        }
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
            }

            if wardrobe_resources.is_clothes_changable {
                ui.label("head");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.head.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.head_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.head) {
                                return;
                            }
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.head = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.head = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "head".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.head));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.head.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.head_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.head.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.head = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.head = new_index.to_string();

                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "head".to_string(), new_index.to_string());
                    }
                });
                ui.label("hair");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.hair.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.hair_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.hair) {
                                return;
                            }
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.hair = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.hair = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "hair".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.hair));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.hair.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.hair_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.hair.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.hair = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.hair = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "hair".to_string(), new_index.to_string());
                    }
                });
                ui.label("eyes");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.eyes.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.eyes_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.eyes) {
                                return;
                            }
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.eyes = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.eyes = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "eyes".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.eyes));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.eyes.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.eyes_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.eyes.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.eyes = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.eyes = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "eyes".to_string(), new_index.to_string());
                    }
                });
                ui.label("upper dress");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.upper_dress.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.upper_dress_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.upper_dress) {
                                return;
                            }
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.upper_dress = new_index.to_string();
                        wardrobe_resources.wardrobe_parts.body_parts.body = new_index.to_string();
                        wardrobe_resources.wardrobe_parts.body_parts.left_hand = new_index.to_string();
                        wardrobe_resources.wardrobe_parts.body_parts.right_hand = new_index.to_string();

                        let uuid = &room_data.this_user_uuid.to_string();
                        let data = room_data.room_users.get_mut(uuid).unwrap();

                        data.body_parts.upper_dress = new_index.to_string();
                        data.body_parts.body = new_index.to_string();
                        data.body_parts.left_hand = new_index.to_string();
                        data.body_parts.right_hand = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "body".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "left_hand".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "right_hand".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.upper_dress));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.upper_dress.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.upper_dress_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.upper_dress.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.upper_dress = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        let data = room_data.room_users.get_mut(uuid).unwrap();

                        data.body_parts.upper_dress = new_index.to_string();
                        data.body_parts.body = new_index.to_string();
                        data.body_parts.left_hand = new_index.to_string();
                        data.body_parts.right_hand = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "body".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "left_hand".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "right_hand".to_string(), new_index.to_string());
                    }
                });
                ui.label("lower dress");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.hip.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.hip_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.hip) {
                                return;
                            }
                        }
                        wardrobe_resources.wardrobe_parts.body_parts.hip = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.hip = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "hip".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.hip));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.hip.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.hip_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.hip.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.hip = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        room_data.room_users.get_mut(uuid).unwrap().body_parts.hip = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "hip".to_string(), new_index.to_string());
                    }
                });
                ui.label("legs");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.legs.parse::<i8>().unwrap();
                        let mut new_index = (index - 1).to_string();

                        if index - 1 < 0 {
                            new_index = (wardrobe_resources.wardrobe_parts.legs_total - 1).to_string();
                            if new_index.eq(&wardrobe_resources.wardrobe_parts.body_parts.legs) {
                                return;
                            }
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.legs = new_index.to_string();
                        wardrobe_resources.wardrobe_parts.body_parts.left_leg = new_index.to_string();
                        wardrobe_resources.wardrobe_parts.body_parts.right_leg = new_index.to_string();

                        let uuid = &room_data.this_user_uuid.to_string();
                        let data = room_data.room_users.get_mut(uuid).unwrap();

                        data.body_parts.legs = new_index.to_string();
                        data.body_parts.left_leg = new_index.to_string();
                        data.body_parts.right_leg = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "left_leg".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "right_leg".to_string(), new_index.to_string());
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut wardrobe_resources.wardrobe_parts.body_parts.legs));
                    if ui.button(">").clicked() {
                        let index = wardrobe_resources.wardrobe_parts.body_parts.legs.parse::<i8>().unwrap();
                        let new_index = ((index + 1) % wardrobe_resources.wardrobe_parts.legs_total).to_string();

                        if wardrobe_resources.wardrobe_parts.body_parts.legs.eq(&new_index) {
                            return;
                        }

                        wardrobe_resources.wardrobe_parts.body_parts.legs = new_index.to_string();
                        let uuid = &room_data.this_user_uuid.to_string();
                        let data = room_data.room_users.get_mut(uuid).unwrap();

                        data.body_parts.legs = new_index.to_string();
                        data.body_parts.left_leg = new_index.to_string();
                        data.body_parts.right_leg = new_index.to_string();
                        
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "left_leg".to_string(), new_index.to_string());
                        replace_wardrobe_sprite(&mut commands, &scene.scene_uuid, &mut s3_data, &mut characters_query, &room_data.this_user_uuid, "right_leg".to_string(), new_index.to_string());
                    }
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if wardrobe_resources.is_map_changable {
                ui.label("Current Map");
                ui.horizontal(|ui| {
                    if ui.button("<").clicked() {
                        let index = match map_data.map_name.parse::<i32>() {
                            Ok(n) => n,
                            Err(_) => 0,
                        };
                        let current_map_name = map_data.map_name.to_string();
                        if index - 1 < 0 {
                            map_data.map_name = (wardrobe_resources.map_total - 1).to_string();
                        } else {
                            map_data.map_name = (index - 1).to_string();
                        }
                        if current_map_name.ne(&map_data.map_name) {
                            map_data.is_map_loaded = false;
                            load_tilemap_from_s3(&mut commands, &scene.scene_uuid, &mut s3_data, "shalloville".to_string(), map_data.map_name.to_string());   
                        }
                    }
                    ui.add_enabled(false, egui::TextEdit::singleline(&mut map_data.map_name));
                    if ui.button(">").clicked() {
                        let index = match map_data.map_name.parse::<i32>() {
                            Ok(n) => n,
                            Err(_) => 0,
                        };
                        let current_map_name = map_data.map_name.to_string();
                        if index + 1 > wardrobe_resources.map_total - 1 {
                            map_data.map_name = String::from('0');
                        } else {
                            map_data.map_name = (index + 1).to_string();
                        }
                        if current_map_name.ne(&map_data.map_name) {
                            map_data.is_map_loaded = false;
                            load_tilemap_from_s3(&mut commands, &scene.scene_uuid, &mut s3_data, "shalloville".to_string(), map_data.map_name.to_string());   
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
            }

            if wardrobe_resources.display_comfirm_button {
                if ui.button(wardrobe_resources.on_confirm_text.to_string()).clicked() {

                    match wardrobe_resources.on_confirm_action {
                        OnWardrobeConfirmed::CreateRoom => {
                            room_data.set_loaded_to_ready();

                            create_room(&mut room_data, &mut rtc_resource, &map_data);

                            create_scene(&mut commands, &mut create_scene_event, &mut room_data, "game", HashMap::from([
                                (StateName::TriggerButtonState as u8, 1),
                                (StateName::MovementState as u8, 1),
                                (StateName::CharacterExistState as u8, 1),
                                (StateName::CameraState as u8, 2),
                                (StateName::MultiplayerRoomState as u8, 1)
                            ]), Some(Vec::from([
                                StateName::WardrobeState as u8,
                                StateName::StreamingState as u8
                            ])));
                        },
                        OnWardrobeConfirmed::JoinRoom => {
                            room_data.set_loaded_to_ready();
                            
                            join_room(&mut commands, &scene.scene_uuid, &mut room_data, &mut rtc_resource);
                        }
                        _ => {}
                    }
    
                    wardrobe_state.set(WardrobeState::Close);
                }
            }
        });
    });   
}

fn replace_wardrobe_sprite(commands: &mut Commands, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, characters_query: &mut Query<&mut Character>, user_uuid: &str, part: String, index: String) {
    for mut character in characters_query.iter_mut() {
        if character.uuid.eq(&user_uuid) {
            if let Some((entity, _)) = character.entity_parts.get(&format!("{} sprite", part)) {
                if let Some((mut _parent, offset)) = character.entity_parts.get(&format!("{} anchor", part)) {
                    commands.entity(*entity).despawn_recursive();
                    let new_offset = offset.clone();
                    let sprite =
                    commands.spawn(
                        SpriteBundle {
                            transform: Transform::from_xyz(0.0, 0.0, new_offset.z),
                            ..default()
                        }
                    ).id();
                    load_sprite_from_s3(commands, scene_uuid, s3_data, "shalloville".to_string(), format!("character/{}/{}.png", part, index), sprite);
                    character.entity_parts.insert(format!("{} sprite", part), (sprite, Vec3::new(0.0, 0.0, new_offset.z)));
                    commands.entity(_parent).add_child(sprite);
                }
            }
            break;
        }
    }
}