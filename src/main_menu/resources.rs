use std::collections::HashMap;

use bevy::prelude::*;

use crate::{character::{resources::CharacterAnimation, systems::create_character}, map_structures::resources::MapData, networking::s3::resources::S3Data, wardrobe::resources::BodyParts};

#[derive(Resource)]
pub struct RoomData {
    pub current_scene: Option<Entity>,
    pub room_id: String,

    pub this_user_name: String,
    pub this_user_uuid: String,

    pub room_users: HashMap<String, UserData>
}

#[derive(PartialEq, Eq, Clone)]
pub enum UserStatus {
    Wait,
    Ready,
    Loaded
}

impl Default for RoomData{
    fn default() -> RoomData {
        RoomData {
            current_scene: None,
            room_id: String::new(),
            this_user_name: String::new(),
            this_user_uuid: String::new(),
            room_users: HashMap::new()
        }
    }
}

pub struct MainMenuImages {
    pub background: Handle<Image>,
    pub canvas: Handle<Image>,
    pub icon: Handle<Image>
}

impl FromWorld for MainMenuImages {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            background: asset_server.load("main_menu/background.png"),
            canvas: asset_server.load("main_menu/canvas.png"),
            icon: asset_server.load("icon/shalloville.png")
        }
    }
}

pub struct UserData {
    pub uuid: String,
    pub username: String,

    pub body_parts: BodyParts,

    pub init_pos: i32,
    pub character: Option<Entity>,
    pub character_controller: CharacterController,

    pub user_status: UserStatus,
}

pub struct CharacterController {
    pos_x: f32,
    pos_y: f32,

    prev_pos_x: f32,
    prev_pos_y: f32,
    pos_time: f32,

    scale_x: f32,
    scale_y: f32,
    current_animation: String,
    previous_animation: String,
    is_changed: bool,
}

impl Default for CharacterController {
    fn default() -> CharacterController {
        CharacterController {
            pos_x: 0.0, pos_y: 0.0, prev_pos_x: 0.0, prev_pos_y: 0.0, pos_time: 0.0, scale_x: 1.0, scale_y: 1.0, current_animation: "idle".to_string(), previous_animation: "idle".to_string(), is_changed: true
        }
    }
}

impl Clone for CharacterController {
    fn clone(&self) -> CharacterController {
        CharacterController {
            pos_x: self.pos_x, prev_pos_x: 0.0, prev_pos_y: 0.0, pos_time: 0.0, pos_y: self.pos_y, scale_x: self.scale_x, scale_y: self.scale_y, current_animation: self.current_animation.to_string(), previous_animation: self.previous_animation.to_string(), is_changed: true
        }
    }
}

impl CharacterController {
    pub fn get_pos(&self) -> (f32, f32, f32, f32, f32) {
        return (self.pos_x, self.pos_y, self.prev_pos_x, self.prev_pos_y, self.pos_time);
    }
    pub fn get_scale(&self) -> (f32, f32) {
        return (self.scale_x, self.scale_y);
    }
    pub fn get_animation(&self) -> (String, String) {
        return (self.current_animation.to_string(), self.previous_animation.to_string());
    }
    pub fn is_changed(&self) -> bool {
        return self.is_changed;
    }
    pub fn set_pos_x(&mut self, x: f32) {
        self.prev_pos_x = self.pos_x;
        self.pos_x = x;
        self.pos_time = 0.0;
        if self.prev_pos_x.ne(&x) {
            let mut scale_x = 1.0;
            if self.prev_pos_x > x {
                scale_x = -1.0;
            }
            self.set_scale_x(scale_x);
            self.is_changed = true;
        }
    }
    pub fn set_pos_y(&mut self, y: f32) {
        self.prev_pos_y = self.pos_y;
        self.pos_y = y;
        self.pos_time = 0.0;
        if self.prev_pos_y.ne(&y) {
            self.is_changed = true;
        }
    }
    pub fn set_scale_x(&mut self, scale_x: f32) {
        self.scale_x = scale_x;
        if self.scale_x.ne(&scale_x) {
            self.is_changed = true;
        }
    }
    pub fn set_scale_y(&mut self, scale_y: f32) {
        self.scale_y = scale_y;
        if self.scale_x.ne(&scale_y) {
            self.is_changed = true;
        }
    }
    pub fn set_animation(&mut self, animation: &str) {
        self.previous_animation = self.current_animation.to_string();
        self.current_animation = animation.to_string();
        if self.current_animation.ne(&self.previous_animation) {
            self.is_changed = true;
        }
    }
    pub fn set_changed(&mut self, is_changed: bool) {
        self.is_changed = is_changed;
    }
    pub fn set_pos_time(&mut self, t: f32) {
        if t <= 1.0 {
            self.pos_time = t;
            self.is_changed = true;
        }
    }
}

impl UserData {
    pub fn create_empty(uuid: &str, body_parts: BodyParts) -> UserData {
        UserData {
            uuid: uuid.to_string(),
            username: "".to_string(),
            body_parts: body_parts,

            init_pos: -1,
            character: None,
            character_controller: CharacterController::default(),

            user_status: UserStatus::Ready
        }
    }

    pub fn remove_character(&mut self, commands: &mut Commands) {
        if let Some(character_entity) = self.character {
            commands.entity(character_entity).despawn_recursive();
        }
    }
}

impl Clone for UserData {
    fn clone(&self) -> UserData {
        UserData {
            uuid: self.uuid.to_string(),
            username: self.username.to_string(),
            body_parts: self.body_parts.clone(),

            init_pos: self.init_pos,
            character: None,
            character_controller: self.character_controller.clone(),

            user_status: self.user_status.clone()
        }
    }
}

impl RoomData {
    pub fn load_ready_users(&mut self, commands: &mut Commands, assets_server: &Res<AssetServer>, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, map_data: &mut ResMut<MapData>, character_animation: &mut Res<CharacterAnimation>){
        if let Some(scene_entity) = self.current_scene {
            for (_, user_data) in self.room_users.iter_mut() {
                if user_data.user_status.ne(&UserStatus::Ready) {
                    continue;
                }
                user_data.user_status = UserStatus::Loaded;
                let character = create_character(commands, assets_server, scene_uuid, s3_data, map_data, character_animation, user_data);
                user_data.character = Some(character);
                commands.entity(scene_entity).add_child(character);

            }
        }
    }

    pub fn load_wait_user(&mut self, commands: &mut Commands, assets_server: &Res<AssetServer>, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, map_data: &mut ResMut<MapData>, character_animation: &mut Res<CharacterAnimation>, user_uuid: &str){
        if let Some(scene_entity) = self.current_scene {
            if let Some(user_data) = self.room_users.get_mut(user_uuid) {
                if user_data.user_status.ne(&UserStatus::Wait) {
                    return;
                }
                user_data.user_status = UserStatus::Loaded;
                let character = create_character(commands, assets_server, scene_uuid, s3_data, map_data, character_animation, user_data);
                user_data.character = Some(character);
                commands.entity(scene_entity).add_child(character);
            }
        }
    }

    pub fn set_loaded_to_ready(&mut self){
        for (_, user_data) in self.room_users.iter_mut() {
            if user_data.user_status.ne(&UserStatus::Loaded) {
                continue;
            }
            user_data.user_status = UserStatus::Ready;
            user_data.character = None;

        }
    }
}