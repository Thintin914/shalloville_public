use bevy::prelude::*;

use crate::{main_menu::resources::RoomData, mouse::resources::MouseData};

use super::components::CameraTag;

pub fn create_camera(mut commands: Commands){
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.227, 0.208, 0.259)),
                ..Default::default()
            },
            ..default()
        },
        CameraTag {cam_x: 0.0, cam_y: 0.0}
    ));
}

pub fn move_camera_free(mut camera_query: Query<(&mut Transform, &mut CameraTag, &OrthographicProjection)>, mouse_data: Res<MouseData>, time: Res<Time>){
    
    if let Ok((mut transform, mut camera_tag, camera_projection)) = camera_query.get_single_mut() {
        let mut translate = Vec3::ZERO;
        let camera_width = camera_projection.area.min.x.abs() + camera_projection.area.max.x.abs();
        let camera_height = camera_projection.area.min.y.abs() + camera_projection.area.max.y.abs();


        if mouse_data.mouse_x < 32.0 {
            translate.x = -1.0;
        } else if mouse_data.mouse_x > camera_width - 32.0 {
            translate.x = 1.0;
        }
        if mouse_data.mouse_y < 32.0 {
            translate.y = -1.0;
        } else if mouse_data.mouse_y > camera_height - 32.0 {
            translate.y = 1.0;
        }

        transform.translation += translate * time.delta_seconds() * 200.0;
        camera_tag.cam_x = transform.translation.x;
        camera_tag.cam_y = transform.translation.y;
    }
}

pub fn camera_follow_user(mut camera_query: Query<(&mut Transform, &mut CameraTag)>, room_data: Res<RoomData>){
    if let Some(user_data) = room_data.room_users.get(&room_data.this_user_uuid) {
        if let Some(entity) = user_data.character {
            if let Ok((mut camera, mut camera_tag)) = camera_query.get_single_mut() {
                let current_pos = user_data.character_controller.get_pos();
                camera.translation = Vec3 {x: current_pos.0, y: current_pos.1, z: 1000.0};
                camera_tag.cam_x = current_pos.0;
                camera_tag.cam_y = current_pos.1;
            }
        }
    }
}

pub fn camera_static(mut camera_query: Query<(&mut Transform, &mut CameraTag)>){
    if let Ok((mut camera, mut camera_tag)) = camera_query.get_single_mut() {
        camera.translation = Vec3::ZERO;
        camera_tag.cam_x = 0.0;
        camera_tag.cam_y = 0.0;
    }
}