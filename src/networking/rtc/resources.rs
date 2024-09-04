use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use image::RgbaImage;

use std::time::Instant;
use bevy::tasks::AsyncComputeTaskPool;
use flume::{bounded, Receiver, Sender};
use livekit::prelude::*;
use livekit::prelude::DataPacketKind;

use livekit_api::services::room::UpdateParticipantOptions;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use std::collections::HashMap;
use std::{env, sync::Arc};
use parking_lot::Mutex;

use livekit_api::{access_token, services::room::{CreateRoomOptions, RoomClient, SendDataOptions}};

use crate::networking::rtc::components::LoadMetadataTask;
use crate::networking::s3::components::ComputeTask;

use super::components::MultiplayerUserAttribute;
use super::video::DeviceVideoTrack;
use super::video_renderer::{self, VideoRenderer};

#[derive(Resource)]
pub struct RTCResource {
    pub room: Arc<Mutex<Option<Room>>>,
    pub stop_room_sender: Option<flume::Sender<bool>>,
    pub room_event: Option<Receiver<livekit::RoomEvent>>,

    topic_cooldown: HashMap<String, (u128, Option<Instant>)>,

    video_tracks: Arc<Mutex<HashMap<String, DeviceVideoTrack>>>,
    published_video_tracks: HashMap<String, VideoRenderer>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomMetadata {
    pub map: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomSentMessage {
    pub a: String,
    pub b: String
}

impl RTCResource {

    pub fn new_room(&mut self, room_id: String, user_uuid: String, metadata: Option<String>, user_attribute: MultiplayerUserAttribute) {
        if self.room.lock().is_some() {
            return;
        }

        let room_arc = Arc::clone(&self.room);

        let (stop_room_sender, stop_room_receiver) = bounded::<bool>(1);
        self.stop_room_sender = Some(stop_room_sender);

        let (room_data_sender, room_data_receiver) = bounded::<livekit::RoomEvent>(1);
        self.room_event = Some(room_data_receiver);
        
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {

                let (url, api_key, api_secret, https_url) = get_livekit_env();

                let room_service = RoomClient::with_api_key(&https_url, &api_key, &api_secret);

                let mut room_metadata = String::new();
                if let Some(metadata) = metadata {
                    room_metadata = metadata;
                }
                let _ = room_service
                .create_room(&room_id, CreateRoomOptions {
                    empty_timeout: 30,
                    metadata: room_metadata,
                    ..default()
                })
                .await;

                let (room, rx) = connect(&room_id, &user_uuid, &url, &api_key, &api_secret, true).await;

                let _ = room_service.update_participant(&room_id, &user_uuid, UpdateParticipantOptions {
                    attributes: get_user_attribute(user_attribute),
                    ..default()
                }).await;

                *room_arc.lock() = Some(room);
                event(rx, stop_room_receiver, room_data_sender).await;
            });
        });
    }

    #[tokio::main]
    pub async fn enter_wardrobe_if_room_exist(&mut self, commands: &mut Commands, scene_uuid: &str, room_id: &str) {
        let (_, is_exist) = check_room_exist(room_id.to_string()).await;

        let thread_pool = AsyncComputeTaskPool::get();
        let task_entity = commands.spawn_empty().id();
    
        let task = thread_pool.spawn(async move {
            let mut command_queue = CommandQueue::default();

            let mut metadata = ":404";
            if is_exist {
                metadata = ":join_room_to_wardrobe";
            }

            command_queue.push(move |world: &mut World| {
                world
                .entity_mut(task_entity)
                .insert(LoadMetadataTask {
                    metadata: String::from(metadata)
                })
                .remove::<ComputeTask>();
            });
            return command_queue;
        });
        commands.entity(task_entity).insert(ComputeTask((scene_uuid.to_string(), task)));
    }

    pub fn join_existing_room(&mut self, commands: &mut Commands, scene_uuid: &str, room_id: &str, user_uuid: &str, user_attribute: MultiplayerUserAttribute) {
        if self.room.lock().is_some() {
            return;
        }

        let cloned_room_id = room_id.to_string();
        let cloned_user_uuid = user_uuid.to_string();

        let cloned_room_id2 = room_id.to_string();
        let cloned_user_uuid2 = user_uuid.to_string();

        let room_arc = Arc::clone(&self.room);

        let (metadata_sender, metadata_receiver) = bounded::<String>(1);

        let (stop_room_sender, stop_room_receiver) = bounded::<bool>(1);
        self.stop_room_sender = Some(stop_room_sender);

        let (room_data_sender, room_data_receiver) = bounded::<livekit::RoomEvent>(1);
        self.room_event = Some(room_data_receiver);

        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let (url, api_key, api_secret, _) = get_livekit_env();

                let (_, is_exist) = check_room_exist(cloned_room_id.to_string()).await;

                if is_exist {
                    let (room, rx) = connect(&cloned_room_id, &cloned_user_uuid, &url, &api_key, &api_secret, false).await;

                    let _ = get_room_service().update_participant(&cloned_room_id2, &cloned_user_uuid2, UpdateParticipantOptions {
                        attributes: get_user_attribute(user_attribute),
                        ..default()
                    }).await;

                    let _ = metadata_sender.send(room.metadata());    
                    *room_arc.lock() = Some(room);
                    event(rx, stop_room_receiver, room_data_sender).await;
                } else {
                    let _ = metadata_sender.send(":404".to_string());
                }
            });
        });

        if let Ok(metadata) = metadata_receiver.recv() {
            let thread_pool = AsyncComputeTaskPool::get();

            let task_entity = commands.spawn_empty().id();
        
            let task = thread_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();
    
                command_queue.push(move |world: &mut World| {
                    world
                    .entity_mut(task_entity)
                    .insert(LoadMetadataTask {
                        metadata: metadata
                    })
                    .remove::<ComputeTask>();
                });
                return command_queue;
            });
            commands.entity(task_entity).insert(ComputeTask((scene_uuid.to_string(), task)));
        }
    }

    pub fn leave_room(&mut self, room_id: String, user_uuid: String) {
        if self.room.lock().is_none() {
            return;
        }

        if let Some(stop_room_sender) = &self.stop_room_sender {
            let _ = stop_room_sender.send(true);
        }
        self.stop_room_sender = None;
        *self.room.lock() = None;
        self.room_event = None;

        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _ = get_room_service().remove_participant(&room_id, &user_uuid).await;
            });
        });
    }

    pub fn send_message(&mut self, room_len: usize, room_id: &str, user_uuid: &str, topic: &str, message: &str, kind: DataPacketKind) {
        if room_len < 2 {
            return;
        }

        if let Some((cooldown, instant_opt)) = self.topic_cooldown.get_mut(topic) {
            if let Some(instant) = instant_opt {
                if instant.elapsed().as_millis() < *cooldown {
                    return;
                }
            }
            *instant_opt = Some(Instant::now());
        }

        let cloned_room_id = room_id.to_string();
        let cloned_topic = topic.to_string();
        let cloned_message = message.to_string();
        let cloned_user_uuid = user_uuid.to_string();

        if let Ok(data) = serde_json::to_string(&RoomSentMessage {
            a: cloned_user_uuid,
            b: cloned_message
        }) {
            std::thread::spawn(move || {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let room_service = get_room_service();
                    let _ = room_service.send_data(&cloned_room_id, data.into_bytes(), SendDataOptions {
                        kind: kind.into(),
                        topic: Some(cloned_topic),
                        ..default()
                    }).await;
                });
            });
        }
    }

    pub fn is_multiplayer(&self) -> bool {
        if self.room.lock().is_some() {
            return true;
        }
        return false;
    }

    pub fn is_video_track_exists(&self, track_name: &str) -> bool {
        if self.video_tracks.lock().contains_key(track_name) {
            return true;
        }
        return false;
    }

    pub fn new_video_track(&mut self, track_name: &str, room_len: usize, image_receiver: flume::Receiver<RgbaImage>){
        // if room_len < 2 {
        //     return;
        // }
        if self.room.lock().is_none(){
            return;
        }
        if self.video_tracks.lock().contains_key(track_name) {
            return;
        }

        let cloned_track_name = track_name.to_string();
        let cloned_room = Arc::clone(&self.room);
        let video_tracks_arc = Arc::clone(&self.video_tracks);
        std::thread::spawn( move || {
            let mut track_lock = video_tracks_arc.lock();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let mut video_track = DeviceVideoTrack::new(cloned_room);
                video_track.publish(&cloned_track_name, image_receiver).await;
                track_lock.insert(cloned_track_name, video_track);
            });
        });
    }

    pub fn close_video_track(&mut self, track_name: &str) {
        if !self.video_tracks.lock().contains_key(track_name) {
            return;
        }

        let cloned_track_name = track_name.to_string();
        let video_tracks_arc = Arc::clone(&self.video_tracks);
        std::thread::spawn( move || {
            let mut track_lock = video_tracks_arc.lock();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Some(video_track) = track_lock.get_mut(&cloned_track_name) {
                    video_track.unpublish().await;
                    track_lock.remove(&cloned_track_name);
                }
            });
        });
    }

    pub fn new_published_video_track(&mut self, track_name: &str, video_renderer: VideoRenderer) {
        if self.room.lock().is_none(){
            return;
        }

        if self.published_video_tracks.contains_key(track_name) {
            return;
        }

        self.published_video_tracks.insert(track_name.to_string(), video_renderer);
    }

    pub fn close_published_video_track(&mut self, track_name: &str) {
        if let Some(video_track) = self.published_video_tracks.get_mut(track_name) {
            video_track.stop();
        }
        self.published_video_tracks.remove(track_name);
    }
}

impl Default for RTCResource {
    fn default() -> RTCResource {
        RTCResource {
            room: Arc::new(Mutex::new(None)),
            stop_room_sender: None,
            room_event: None,

            topic_cooldown: HashMap::from([
                ("move".to_string(), (200, None))
            ]),

            video_tracks: Arc::new(Mutex::new(HashMap::new())),
            published_video_tracks: HashMap::new()
        }
    }
}

async fn connect(room_id: &str, user_uuid: &str, url: &str, api_key: &str, api_secret: &str, admin: bool) -> (livekit::Room, UnboundedReceiver<livekit::RoomEvent>) {
    let token = access_token::AccessToken::with_api_key(&api_key, &api_secret)
    .with_identity(&user_uuid)
    .with_name(&user_uuid)
    .with_grants(access_token::VideoGrants {
        room_join: true,
        room_admin: admin,
        room: room_id.to_string(),
        ..Default::default()
    })
    .to_jwt()
    .unwrap();

    let (room, rx) = Room::connect(&url, &token, RoomOptions {
        auto_subscribe: true,
        ..default()
    })
    .await
    .unwrap();

    return (room, rx);
}

async fn check_room_exist(room_id: String) -> (Option<RoomClient>, bool) {
    let (_, api_key, api_secret, https_url) = get_livekit_env();

    let room_service = RoomClient::with_api_key(&https_url, &api_key, &api_secret);

    if let Ok(rooms) = room_service.list_rooms(vec![room_id.to_string()]).await {
        if rooms.len().eq(&0) {
            return (None, false);
        }
        return (Some(room_service), true);
    }
    return (None, false);
}

fn get_livekit_env() -> (String, String, String, String){
    let url = env::var("LIVEKIT_URL").expect("LIVEKIT_URL is not set");
    let api_key = env::var("LIVEKIT_API_KEY").expect("LIVEKIT_API_KEY is not set");
    let api_secret = env::var("LIVEKIT_API_SECRET").expect("LIVEKIT_API_SECRET is not set");

    let mut https_url = url.to_string();
    if https_url.starts_with("wss") {
        https_url = https_url.replace("wss", "https");
    }

    return (url, api_key, api_secret, https_url);
}

fn get_room_service() -> RoomClient{
    let (_, api_key, api_secret, https_url) = get_livekit_env();
    let room_service = RoomClient::with_api_key(&https_url, &api_key, &api_secret);
    return room_service;
}

async fn event(mut room_rx: UnboundedReceiver<RoomEvent>, stop_room_receiver: Receiver<bool>, room_data_sender: Sender<livekit::RoomEvent>) {
    async_std::task::block_on(async {
        loop {
            if let Ok(stop) = stop_room_receiver.try_recv() {
                if stop {
                    drop(stop_room_receiver);
                    drop(room_data_sender);    
                    println!("stop loop");
                    break;
                }
            }
    
            if let Ok(room_event) = room_rx.try_recv() {
                let _ = room_data_sender.send(room_event);
            }
        }
    });
}

fn get_user_attribute(user_attribute: MultiplayerUserAttribute) -> HashMap<String, String> {
    return HashMap::from([
        (String::from("name"), user_attribute.username),
        (String::from("hair"), user_attribute.body_part.hair),
        (String::from("eyes"), user_attribute.body_part.eyes),
        (String::from("head"), user_attribute.body_part.head),
        (String::from("upper"), user_attribute.body_part.upper_dress),
        (String::from("hip"), user_attribute.body_part.hip),
        (String::from("legs"), user_attribute.body_part.legs)
    ]);
}