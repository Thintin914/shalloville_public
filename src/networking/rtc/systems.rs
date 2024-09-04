use bevy::{prelude::*, utils::tracing::Instrument};
use livekit::track::{LocalTrack, RemoteTrack};
use uuid::Uuid;

use std::collections::HashMap;

use crate::{character::{resources::CharacterAnimation, systems::push_character}, editor::{resources::{Scene, StateName}, systems::create_scene, CreateSceneEvent}, game::resources::GameResources, main_menu::resources::{RoomData, UserData, UserStatus}, map_structures::resources::MapData, networking::s3::resources::S3Data, nokhwa::resources::{NokhwaCamera, StreamingResources}, utils::group_numbers, wardrobe::resources::{BodyParts, OnWardrobeConfirmed, WardrobeResources}};

use super::{components::{LoadMetadataTask, MultiplayerUserAttribute}, video_renderer::VideoRenderer, RTCResource, RoomMetadata, RoomSentMessage};

pub fn create_room(room_data: &mut ResMut<RoomData>, rtc_resource: &mut ResMut<RTCResource>, map_data: &ResMut<MapData>){
    room_data.room_id = Uuid::new_v4().to_string();

    let room_metadata = RoomMetadata {
        map: map_data.map_name.to_string()
    };
    let room_metadata_str = serde_json::to_string(&room_metadata).unwrap();

    let user_data = room_data.room_users.get(&room_data.this_user_uuid).unwrap();
    let user_attribute = MultiplayerUserAttribute {
        username: user_data.username.to_string(),
        body_part: user_data.body_parts.clone()
    };

    rtc_resource.new_room(room_data.room_id.to_string(), room_data.this_user_uuid.to_string(), Some(room_metadata_str), user_attribute);
}

pub fn check_room(commands: &mut Commands, scene_uuid: &str, rtc_resource: &mut ResMut<RTCResource>, room_id: String){
    rtc_resource.enter_wardrobe_if_room_exist(commands, &scene_uuid, &room_id);
}

pub fn join_room(commands: &mut Commands, scene_uuid: &str, room_data: &mut ResMut<RoomData>, rtc_resource: &mut ResMut<RTCResource>){
    let user_data = room_data.room_users.get(&room_data.this_user_uuid).unwrap();
    let user_attribute = MultiplayerUserAttribute {
        username: user_data.username.to_string(),
        body_part: user_data.body_parts.clone()
    };

    let _ = rtc_resource.join_existing_room(commands, &scene_uuid, &room_data.room_id, &room_data.this_user_uuid, user_attribute);
}

pub fn leave_room(room_data: &mut ResMut<RoomData>, rtc_resource: &mut ResMut<RTCResource>){
    rtc_resource.leave_room(room_data.room_id.to_string(), room_data.this_user_uuid.to_string());

    room_data.room_id.clear();
    room_data.room_users.clear();
}

pub fn load_room_metadata_event_listener(events_query: Query<(Entity, &LoadMetadataTask)>, scene: Res<Scene>, mut commands: Commands, mut room_data: ResMut<RoomData>, mut map_data: ResMut<MapData>, mut wardrobe_resources: ResMut<WardrobeResources>, mut create_scene_event: EventWriter<CreateSceneEvent>){
    for (entity, load_metadata_task) in events_query.iter() {

        if load_metadata_task.metadata.starts_with(':') {
            match load_metadata_task.metadata.as_str() {
                ":404" => {
                    room_data.room_id.clear();

                    if scene.scene_name.ne("lobby") {
                        create_scene(&mut commands, &mut create_scene_event, &mut room_data, "lobby", HashMap::from([
                            (StateName::MainMenuState as u8, 2),
                            (StateName::RoomMetadataListener as u8, 1)
                        ]), None);
                    }
                },
                ":join_room_to_wardrobe" => {
                    wardrobe_resources.is_name_changable = true;
                    wardrobe_resources.is_clothes_changable = true;
                    wardrobe_resources.is_map_changable = false;
                    wardrobe_resources.display_comfirm_button = true;
                    wardrobe_resources.on_confirm_text = "Join Room".to_string();
                    wardrobe_resources.on_confirm_action = OnWardrobeConfirmed::JoinRoom;
    
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
                _ => {}
            }
            commands.entity(entity).despawn();
            return;
        }

        if let Ok(metadata) = serde_json::from_str::<RoomMetadata>(&load_metadata_task.metadata) {
            map_data.map_name = metadata.map;

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
        }

        commands.entity(entity).despawn();
    }
}

pub fn on_room_event_received(mut commands: Commands, assets_server: Res<AssetServer>, mut rtc_resource: ResMut<RTCResource>, scene: Res<Scene>, mut room_data: ResMut<RoomData>, mut s3_data: ResMut<S3Data>, mut map_data: ResMut<MapData>, mut character_animation: Res<CharacterAnimation>, mut game_resource: ResMut<GameResources>, mut nokhwa_camera: Res<NokhwaCamera>){
    if let Some(room_event) = &rtc_resource.room_event {
        if let Ok(rtc_room_event) = room_event.try_recv() {
            println!("----------");
            match rtc_room_event {
                livekit::RoomEvent::Connected { participants_with_tracks } => {
                    for (remote_participant, remote_track) in participants_with_tracks.iter() {
                        if remote_participant.name().eq(&room_data.this_user_uuid) {
                            continue;
                        }
                        if room_data.room_users.contains_key(&remote_participant.name()) {
                            continue;
                        }
                        for track in remote_track.iter() {
                            track.set_subscribed(true);
                        }
                        let mut user_data = UserData::create_empty(&remote_participant.name(), BodyParts::default());
                        set_attributes(&mut user_data, &remote_participant.attributes());
                        push_character(&mut commands, &assets_server, &scene.scene_uuid, &mut s3_data, &mut map_data, &mut room_data, &mut character_animation, &mut user_data);
                    }
                },
                livekit::RoomEvent::ParticipantDisconnected(remote_participant) => {
                    if let Some(user_data) = room_data.room_users.get_mut(&remote_participant.name()) {
                        user_data.remove_character(&mut commands);
                        room_data.room_users.remove(&remote_participant.name());
                    }
                    for (_, publication) in remote_participant.track_publications() {
                        publication.set_subscribed(false);
                    }
                },
                livekit::RoomEvent::ParticipantConnected(remote_participant) => {
                    let mut user_data = UserData::create_empty(&remote_participant.name(), BodyParts::default());
                    if remote_participant.attributes().len().eq(&0) {
                        user_data.user_status = UserStatus::Wait;
                    } else {
                        set_attributes(&mut user_data, &remote_participant.attributes());
                    }
                    push_character(&mut commands, &assets_server, &scene.scene_uuid, &mut s3_data, &mut map_data, &mut room_data, &mut character_animation, &mut user_data);

                    if nokhwa_camera.has_opened {
                        if !rtc_resource.is_video_track_exists(&room_data.this_user_uuid) {
                            if let Some(image_receiver) = &nokhwa_camera.image_receiver {
                                rtc_resource.new_video_track(&room_data.this_user_uuid, room_data.room_users.len(), image_receiver.clone());
                            }
                        }
                    }
                },
                livekit::RoomEvent::ParticipantAttributesChanged { participant, changed_attributes } => {
                    if participant.name().eq(&room_data.this_user_uuid) {
                        return;
                    }
                    let user_uuid = participant.name().to_string();

                    if let Some(user_data) = room_data.room_users.get_mut(&user_uuid) {
                        set_attributes(user_data, &changed_attributes);
                        room_data.load_wait_user(&mut commands, &assets_server,  &scene.scene_uuid, &mut s3_data, &mut map_data, &mut character_animation, &user_uuid);
                    }
                }
                livekit::RoomEvent::DataReceived { payload, topic, kind, participant } => {
                    if let Ok(msg) = String::from_utf8((*payload).to_vec()) {
                        if let Ok(data) = serde_json::from_str::<RoomSentMessage>(&msg) {
                            if data.a.eq(&room_data.this_user_uuid) {
                                return;
                            }

                            if let Some(user_data) = room_data.room_users.get_mut(&data.a) {
                                if let Some(topic) = topic {
                                    match topic.as_str() {
                                        "chat" => {
                                            game_resource.chat_messages.push(data.b);
                                        },
                                        "move" => {
                                            if let Ok(groups) = group_numbers(data.b, 2) {
                                                let x = groups[0][0];
                                                let y = groups[0][1];
                                                user_data.character_controller.set_pos_x(x);
                                                user_data.character_controller.set_pos_y(y);
                                                user_data.character_controller.set_animation("walk");
                                            }
                                        },
                                        "anime" => {
                                            let groups = data.b.split(' ').collect::<Vec<&str>>();
                                            let anime = groups[0];
                                            let x = groups[1].parse::<f32>().unwrap();
                                            let y = groups[2].parse::<f32>().unwrap();
                                            user_data.character_controller.set_animation(&anime);
                                            user_data.character_controller.set_pos_x(x);
                                            user_data.character_controller.set_pos_y(y);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                },
                // livekit::RoomEvent::LocalTrackPublished { publication, track, participant } => {
                    // if let LocalTrack::Video(ref video_track) = track {
                    //     let video_renderer = VideoRenderer::new(
                    //         video_track.rtc_track()
                    //     );
                    //     println!("published: {}", publication.name());
                    //     rtc_resource.new_published_video_track(&publication.name(), video_renderer);
                    // }
                // },
                // livekit::RoomEvent::LocalTrackUnpublished { publication, participant } => {
                //     println!("unpublished: {}", publication.name());
                //     rtc_resource.close_published_video_track(&publication.name());
                // },
                _ => {
                    println!("{:?}", rtc_room_event);
                }
            }
        }
    }
}

fn set_attributes(user_data: &mut UserData, changed_attributes: &HashMap<String, String>){
    println!("set user attributes");
    for (key, v) in changed_attributes.iter() {
        let value = v.to_string();
        match key.as_str() {
            "name" => user_data.username = value,
            "hair" => user_data.body_parts.hair = value,
            "eyes" => user_data.body_parts.eyes = value,
            "head" => user_data.body_parts.head = value,
            "upper" => {
                user_data.body_parts.upper_dress = value.to_string();
                user_data.body_parts.left_hand = value.to_string();
                user_data.body_parts.right_hand = value.to_string();
                user_data.body_parts.body = value.to_string();
            },
            "hip" => user_data.body_parts.hip = value,
            "legs" => {
                user_data.body_parts.legs = value.to_string();
                user_data.body_parts.left_leg = value.to_string();
                user_data.body_parts.right_leg = value.to_string();
            }
            _ => {}
        }
    }
}