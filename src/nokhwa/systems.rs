use bevy::prelude::*;
use crate::{editor::resources::Scene, main_menu::resources::RoomData, map_structures::{components::{InteractiveEvent, InteractiveType}, resources::{MapData, MapStructures}}, networking::rtc::resources::RTCResource, utils::group_numbers};

use super::{NokhwaCamera, StreamingResources, StreamingState};

pub fn detect_device_camera(mut commands: Commands, scene: Res<Scene>, mut image_assets: ResMut<Assets<Image>>, mut interactive_event: EventReader<InteractiveEvent>, mut nokhwa_camera: ResMut<NokhwaCamera>, mut streaming_resources: ResMut<StreamingResources>, map_data: Res<MapData>, map_structures: Res<MapStructures>, mut streaming_state: ResMut<NextState<StreamingState>>, mut rtc_resource: ResMut<RTCResource>, room_data: Res<RoomData>){
    for ev in interactive_event.read(){
        if ev.0.0.ne(&InteractiveType::SwitchCameraShare) {
            break;
        }

        nokhwa_camera.switch_camera();
        if nokhwa_camera.has_opened {
            streaming_state.set(StreamingState::Open);
            streaming_resources.add_screen(&format!("{} cam", room_data.this_user_uuid));
            if let Some(info) = map_structures.info.get(&ev.0.1.to_string()) {
                for (title, content) in info.iter() {
                    if title.eq(&"screens") {
                        if let Ok(groups) = group_numbers(content.to_string(), 3) {
                            for group in groups.iter() {
                                streaming_resources.add_sprite(&mut commands, &scene.scene_uuid, &format!("{} cam", room_data.this_user_uuid), Vec3::new(group[0], group[1], map_data.max_y + 10.0), group[2]);
                            }
                        }
                    }
                }
            }

            if rtc_resource.is_multiplayer() {
                if let Some(image_receiver) = &nokhwa_camera.image_receiver {
                    rtc_resource.new_video_track(&room_data.this_user_uuid, room_data.room_users.len(), image_receiver.clone());
                }
            }
        } else {
            streaming_state.set(StreamingState::Close);
            streaming_resources.remove_screen(&mut commands, &mut image_assets, &format!("{} cam", room_data.this_user_uuid));

            if rtc_resource.is_multiplayer() {
                rtc_resource.close_video_track(&room_data.this_user_uuid);
            }
        }
    }
}

pub fn update_device_camera_image(mut image_assets: ResMut<Assets<Image>>, mut sprite_query: Query<&mut Handle<Image>>, nokhwa_camera: Res<NokhwaCamera>, mut streaming_resources: ResMut<StreamingResources>, room_data: Res<RoomData>){
    if !nokhwa_camera.has_opened {
        return;
    }

    if let Some(image_receiver) = &nokhwa_camera.image_receiver {
        if let Ok(image) = image_receiver.try_recv() {
            streaming_resources.update_screen(&mut image_assets, &format!("{} cam", room_data.this_user_uuid), image, &mut sprite_query);
        }
    }
}

pub fn close_device_camera(mut commands: Commands, mut nokhwa_camera: ResMut<NokhwaCamera>, mut streaming_resources: ResMut<StreamingResources>, mut rtc_resource: ResMut<RTCResource>, room_data: Res<RoomData>){
    if nokhwa_camera.has_opened {
        nokhwa_camera.switch_camera();

        if rtc_resource.is_multiplayer() {
            rtc_resource.close_video_track(&room_data.this_user_uuid);
        }
    }
    streaming_resources.remove_all(&mut commands);
}