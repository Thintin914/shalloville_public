use std::collections::HashMap;

use bevy::prelude::*;
use livekit::DataPacketKind;

use crate::{editor::{resources::Scene, TilemapLoadedEvent}, game::{components::{Depth, TriggerButton}, MultiplayerRoomState, TriggerButtonState}, main_menu::resources::{RoomData, UserData, UserStatus}, map_structures::{components::InteractiveTrigger, resources::{MapData, MapStructures}}, networking::{rtc::resources::RTCResource, s3::{resources::S3Data, systems::load_sprite_from_s3}}, utils::{from_grid_xy_to_index, from_index_to_xy, from_xy_to_grid, lerp}, wardrobe::resources::BodyParts, AnimationTimer};

use super::{components::Character, CharacterAnimation, TrackLoop, MAX_FRAME};

pub fn push_character(commands: &mut Commands, assets_server: &Res<AssetServer>, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, map_data: &mut ResMut<MapData>, room_data: &mut ResMut<RoomData>, character_animation: &mut Res<CharacterAnimation>, user_data: &mut UserData) -> Option<Entity> {
    if !map_data.is_map_loaded || user_data.user_status.eq(&UserStatus::Wait) {    
        let user_uuid = user_data.uuid.to_string(); 
        room_data.room_users.insert(user_uuid, user_data.clone());
        return None;
    }

    if user_data.user_status.eq(&UserStatus::Ready) {
        if let Some(character_entity) = user_data.character {
            commands.entity(character_entity).despawn_recursive();
        }

        let mut cloned_user_data = user_data.clone();
        let character = create_character(commands, assets_server, &scene_uuid, s3_data, map_data, character_animation, &mut cloned_user_data);
        cloned_user_data.character = Some(character);
        room_data.room_users.insert(cloned_user_data.uuid.to_string(), cloned_user_data);
        commands.entity(room_data.current_scene.unwrap()).add_child(character);
        return Some(character);
    }

    return None;
}

pub fn on_tilemap_loaded(mut tilemap_loaded_event: EventReader<TilemapLoadedEvent>, scene: Res<Scene>, mut commands: Commands, assets_server: Res<AssetServer>, mut s3_data: ResMut<S3Data>, mut map_data: ResMut<MapData>, mut room_data: ResMut<RoomData>, mut character_animation: Res<CharacterAnimation>) {
    for _ in tilemap_loaded_event.read() {
        room_data.load_ready_users(&mut commands, &assets_server,&scene.scene_uuid, &mut s3_data, &mut map_data, &mut character_animation);

        // set position of loaded characters to map spawn position
        for (_uuid, user_data) in room_data.room_users.iter_mut() {
            user_data.init_pos = map_data.spawn_pos;
            let (x, y) = from_index_to_xy(user_data.init_pos, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
            user_data.character_controller.set_pos_x(x as f32);
            user_data.character_controller.set_pos_y(y as f32);
        }
    }
}

fn create_character_part(commands: &mut Commands, assets_server: &Res<AssetServer>, scene_uuid: &str, character_animation: &mut Res<CharacterAnimation>, s3_data: &mut ResMut<S3Data>, part_name: &str, part_index: &str) -> (Entity, Entity, Vec3) {
    let offset = character_animation.character_offset.get(part_name).unwrap();
    let anchor = commands.spawn(
        SpatialBundle {
            transform: Transform::from_xyz(offset.x, offset.y, offset.z),
            ..default()
        }
    ).id();
    let sprite = commands.spawn(
        SpriteBundle {
            texture: assets_server.load(format!("placeholder/{}.png", part_name)),
            transform: Transform::from_xyz(0.0, 0.0, offset.z),
            ..default()
        },
    ).id();

    load_sprite_from_s3(commands, scene_uuid, s3_data, "shalloville".to_string(), format!("character/{}/{}.png", part_name, part_index), sprite);
    commands.entity(anchor).add_child(sprite);

    return (anchor, sprite, *offset)
}

pub fn create_character(commands: &mut Commands, assets_server: &Res<AssetServer>, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, map_data: &mut ResMut<MapData>, character_animation: &mut Res<CharacterAnimation>, user_data: &mut UserData) -> Entity{

    let mut init_pos = user_data.init_pos;
    if init_pos.eq(&-1) {
        init_pos = map_data.spawn_pos;
    }

    let (pos_x, pos_y) = from_index_to_xy(init_pos, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
    let container = commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(pos_x as f32, pos_y as f32, 1.0),
            ..default()
        },
        Depth {
            index: map_data.max_y - pos_y as f32
        }
    )).id();

    let (left_leg_anchor, left_leg_sprite, left_leg_offset) = create_character_part(commands, assets_server, scene_uuid, character_animation, s3_data, "left_leg", &user_data.body_parts.left_leg);
    let (right_leg_anchor, right_leg_sprite, right_leg_offset) = create_character_part(commands, assets_server,scene_uuid, character_animation, s3_data, "right_leg", &user_data.body_parts.right_leg);
    let (left_hand_anchor, left_hand_sprite, left_hand_offset) = create_character_part(commands, assets_server,scene_uuid, character_animation, s3_data, "left_hand", &user_data.body_parts.left_hand);
    let (body_anchor, body_sprite, body_offset) = create_character_part(commands, assets_server,scene_uuid, character_animation, s3_data, "body", &user_data.body_parts.body);
    let (hip_anchor, hip_sprite, hip_offset) = create_character_part(commands, assets_server,scene_uuid, character_animation, s3_data, "hip", &user_data.body_parts.hip);
    let (right_hand_anchor, right_hand_sprite, right_hand_offset) = create_character_part(commands, assets_server, scene_uuid, character_animation, s3_data, "right_hand", &user_data.body_parts.right_hand);
    let (head_anchor, head_sprite, head_offset) = create_character_part(commands,assets_server, scene_uuid, character_animation, s3_data, "head", &user_data.body_parts.head);
    let (eyes_anchor, eyes_sprite, eyes_offset) = create_character_part(commands, assets_server, scene_uuid, character_animation, s3_data, "eyes", &user_data.body_parts.eyes);
    let (hair_anchor, hair_sprite, hair_offset) = create_character_part(commands, assets_server, scene_uuid,character_animation, s3_data, "hair", &user_data.body_parts.hair);

    let hip_group = 
    commands.entity(hip_anchor).add_child(body_anchor).add_child(left_leg_anchor).add_child(right_leg_anchor).id();
    commands.entity(body_anchor).add_child(left_hand_anchor).add_child(right_hand_anchor).add_child(head_anchor);
    commands.entity(head_anchor).add_child(eyes_anchor).add_child(hair_anchor);

    commands.entity(container).insert((
        SpatialBundle {
            transform: Transform::from_xyz(pos_x as f32, pos_y as f32, 1.0),
            ..default()
        },
        Character {
            uuid: user_data.uuid.to_string(), current_frame: 0, increment: 1,
             entity_parts: HashMap::from([
                ("head anchor".to_string(), (head_anchor, head_offset)),
                ("eyes anchor".to_string(), (eyes_anchor, eyes_offset)),
                ("hair anchor".to_string(), (hair_anchor, hair_offset)),
                ("body anchor".to_string(), (body_anchor, body_offset)),
                ("left_hand anchor".to_string(), (left_hand_anchor, left_hand_offset)),
                ("right_hand anchor".to_string(), (right_hand_anchor, right_hand_offset)),
                ("hip anchor".to_string(), (hip_anchor, hip_offset)),
                ("left_leg anchor".to_string(), (left_leg_anchor, left_leg_offset)),
                ("right_leg anchor".to_string(), (right_leg_anchor, right_leg_offset)),
                
                ("head sprite".to_string(), (head_sprite, Vec3::new(0.0, 0.0, head_offset.z))),
                ("eyes sprite".to_string(), (eyes_sprite, Vec3::new(0.0, 0.0, eyes_offset.z))),
                ("hair sprite".to_string(), (hair_sprite, Vec3::new(0.0, 0.0, hair_offset.z))),
                ("body sprite".to_string(), (body_sprite, Vec3::new(0.0, 0.0, head_offset.z))),
                ("left_hand sprite".to_string(), (left_hand_sprite, Vec3::new(0.0, 0.0, left_hand_offset.z))),
                ("right_hand sprite".to_string(), (right_hand_sprite, Vec3::new(0.0, 0.0, right_hand_offset.z))),
                ("hip sprite".to_string(), (hip_sprite, Vec3::new(0.0, 0.0, hip_offset.z))),
                ("left_leg sprite".to_string(), (left_leg_sprite, Vec3::new(0.0, 0.0, left_leg_offset.z))),
                ("right_leg sprite".to_string(), (right_leg_sprite, Vec3::new(0.0, 0.0, right_leg_offset.z))),
             ])
        }
    ));

    commands.entity(container).add_child(hip_group);

    return container;
}

pub fn update_character_animation(time: Res<Time>, mut local_timer: Local<AnimationTimer>, mut character_animation: ResMut<CharacterAnimation>, mut characters_query: Query<&mut Character>, mut transform_query: Query<&mut Transform>, room_data: Res<RoomData>){
    local_timer.collasped_timer += time.delta_seconds();

    if local_timer.collasped_timer > 0.1 {
        local_timer.collasped_timer = 0.0;
        for mut character in characters_query.iter_mut() {
            if let Some(user_data) = room_data.room_users.get(&character.uuid) {
                let current_animation = user_data.character_controller.get_animation();
                character.current_frame += character.increment;
                if character.current_frame > MAX_FRAME || character.current_frame < 0 {
                    character.increment *= -1;
                } else {
                    let mut animation = current_animation.0;
                    set_character_transform(&mut animation, "head anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "eyes anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "hair anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "body anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "left_hand anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "right_hand anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "left_leg anchor", &mut character, &mut character_animation, &mut transform_query);
                    set_character_transform(&mut animation, "right_leg anchor", &mut character, &mut character_animation, &mut transform_query);
                    if let Some((total_track, track_loop)) = character_animation.max.get(&animation) {
                        if character.current_frame >= total_track - 1 {
                            match track_loop {
                                TrackLoop::PingPong => {character.increment *= -1},
                                TrackLoop::Restart => {
                                    character.increment = 1;
                                    character.current_frame = 0;
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

fn set_character_transform(animation: &mut String, part: &str, character: &mut Character, character_animation: &mut ResMut<CharacterAnimation>, transform_query: &mut Query<&mut Transform>){
    if let Some(track) = character_animation.animations.get(&format!("{} {}", animation, part)) {
        let (total_track, _) = character_animation.max.get(animation).unwrap();
        if let Some((local, rotation)) = track.hashmap.get(&(character.current_frame % total_track)) {
            let (entity, offset) = character.entity_parts.get(part).unwrap();
            if let Ok(mut transform) = transform_query.get_mut(*entity) {
                transform.translation = *local + *offset;
                transform.rotation = *rotation;
            }
        }
    }
}

fn reset_all_animations(character: &mut Character, transform_query: &mut Query<&mut Transform>) {
    for (_, (entity, offset)) in character.entity_parts.iter_mut() {
        if let Ok(mut transform) = transform_query.get_mut(*entity) {
            transform.translation = *offset;
            transform.rotation = Quat::IDENTITY;
            character.current_frame = 0;
            character.increment = 1;
        }
    }
}

pub fn move_character_on_input(keys: Res<ButtonInput<KeyCode>>, map_data: Res<MapData>, mut room_data: ResMut<RoomData>, tile_storage: Res<MapStructures>, time: Res<Time>, mut rtc_resource: ResMut<RTCResource>) {
    let this_user_uuid = room_data.this_user_uuid.to_string();
    if let Some(user_data) = room_data.room_users.get_mut(&this_user_uuid) {
        let mut translation = Vec3::ZERO;
        let mut is_pressed = false;
        if keys.pressed(KeyCode::KeyW) {
            translation.y = 1.0;
            is_pressed = true;
        } else if keys.pressed(KeyCode::KeyS) {
            translation.y = -1.0;
            is_pressed = true;
        }
        if keys.pressed(KeyCode::KeyD) {
            translation.x = 1.0;
            is_pressed = true;
        } else if keys.pressed(KeyCode::KeyA) {
            translation.x = -1.0;
            is_pressed = true;
        }

        if is_pressed {
            let pos_x = translation.x * time.delta_seconds() * map_data.tile_width * 2.0;
            let pos_y = translation.y * time.delta_seconds() * map_data.tile_width * 2.0;
            let mut current_pos = user_data.character_controller.get_pos();
            let (grid_x, grid_y) = from_xy_to_grid(current_pos.0 + pos_x, current_pos.1 + pos_y, map_data.tile_width, map_data.tile_height);
            if grid_x as i32 >= map_data.map_col || grid_y as i32 >= map_data.map_row || grid_x < 0.0 || grid_y < 0.0 {
                return;
            }
            let pos = from_grid_xy_to_index(grid_x, grid_y, map_data.map_col as f32);
            if let Some((is_hitbox, _)) = tile_storage.hitbox.get(&pos) {
                if *is_hitbox {
                    return;
                }
            }

            user_data.character_controller.set_animation("walk");
            user_data.character_controller.set_pos_x(current_pos.0 + pos_x);
            user_data.character_controller.set_pos_y(current_pos.1 + pos_y);

            current_pos = user_data.character_controller.get_pos();


            if rtc_resource.is_multiplayer() {
                rtc_resource.send_message(room_data.room_users.len(), &room_data.room_id, &room_data.this_user_uuid, "move", &format!("{} {}", current_pos.0, current_pos.1), DataPacketKind::Lossy)
            }
        } else {
            user_data.character_controller.set_animation("idle");
        }
    }
}

pub fn update_character_controller(mut transform_query: Query<&mut Transform>, mut characters_query: Query<(&mut Depth, &mut Character)>, mut room_data: ResMut<RoomData>, mut interactive_triggers_query: Query<(Entity, &InteractiveTrigger)>, map_data: Res<MapData>, mut z_button_query: Query<&mut TriggerButton>, mut trigger_button_state: ResMut<NextState<TriggerButtonState>>, mut rtc_resource: ResMut<RTCResource>){
    let this_user_uuid = room_data.this_user_uuid.to_string();
    let room_id = room_data.room_id.to_string();
    let room_len = room_data.room_users.len();

    for (uuid, user_data) in room_data.room_users.iter_mut() {
        if !user_data.character_controller.is_changed() {
            continue;
        }
        user_data.character_controller.set_changed(false);

        if let Some(character_entity) = user_data.character {

            let pos = user_data.character_controller.get_pos();
            let pos_x = lerp(pos.2, pos.0, pos.4);
            let pos_y = lerp(pos.3, pos.1, pos.4);
            user_data.character_controller.set_pos_time(pos.4 + 0.1);

            if let Ok(mut character_transform) = transform_query.get_mut(character_entity) {
                character_transform.translation.x = pos_x;
                character_transform.translation.y = pos_y;
                character_transform.scale.x = user_data.character_controller.get_scale().0;
            }

            if let Ok((mut depth, mut character)) = characters_query.get_mut(character_entity) {
                depth.index = map_data.max_y - pos.1;

                let current_animation = user_data.character_controller.get_animation();
                if current_animation.0.ne(&current_animation.1) {
                    if this_user_uuid.eq(uuid) && rtc_resource.is_multiplayer() {
                        rtc_resource.send_message(room_len, &room_id, &this_user_uuid, "anime", &format!("{} {} {}", &current_animation.0, pos.0, pos.1), DataPacketKind::Reliable)
                    }
                    reset_all_animations(&mut character, &mut transform_query);
                }
            }

            if uuid.eq(&this_user_uuid) {
                let (trigger_opt, target_pos) = detect_interactive_trigger(&mut transform_query, &mut interactive_triggers_query, map_data.tile_width,  pos_x, pos_y);
                if let Ok(mut z_button) = z_button_query.get_single_mut() {
                    if let Some(trigger) = trigger_opt {
                        trigger_button_state.set(TriggerButtonState::Display);
                        z_button.pos_x = target_pos.x;
                        z_button.pos_y = target_pos.y;
                        z_button.trigger = trigger.trigger;
                    } else {
                        trigger_button_state.set(TriggerButtonState::Hidden);
                    }
                }
            }
        }
    }
}

fn detect_interactive_trigger(transform_query: &mut Query<&mut Transform>, interactive_triggers_query: &mut Query<(Entity, &InteractiveTrigger)>, tile_width: f32, character_pos_x: f32, character_pos_y: f32) -> (Option<InteractiveTrigger>, Vec2) {
    let mut distance = tile_width * 2.0;
    let mut final_trigger: Option<InteractiveTrigger> = None;
    let mut target_pos: Vec2 = Vec2::new(0.0, 0.0);
    for (entity, trigger) in interactive_triggers_query.iter() {
        if let Ok(trigger_transform) = transform_query.get(entity) {
            let dist = ((trigger_transform.translation.x - character_pos_x).powi(2) + (trigger_transform.translation.y - character_pos_y).powi(2)).sqrt();
            if dist < distance {
                distance = dist;
    
                if dist <= tile_width {
                    final_trigger = Some(trigger.clone());
                    target_pos = Vec2::new(trigger_transform.translation.x, trigger_transform.translation.y + 16.0);
                }
            }
        }
    }
    return (final_trigger, target_pos);
}