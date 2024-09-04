use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::AsyncComputeTaskPool;
use image::{DynamicImage, RgbaImage};
use std::time::Duration;
use bevy::utils::Instant;

use flume::bounded;
use nokhwa::pixel_format::{RgbAFormat, RgbFormat};
use nokhwa::utils::{ApiBackend, CameraInfo, RequestedFormat, RequestedFormatType};
use nokhwa::CallbackCamera;
use nokhwa::{nokhwa_initialize, query};

use crate::networking::s3::components::ComputeTask;

#[derive(Resource)]
pub struct StreamingResources {
    pub screens: HashMap<String, Screen>
}

pub struct Screen {
    pub width: Arc<Mutex<f32>>,
    pub height: Arc<Mutex<f32>>,
    pub image_handle: Option<Handle<Image>>,
    pub containers: Vec<Entity>,
    pub screens: Vec<Entity>,
}

impl StreamingResources {
    pub fn add_screen(&mut self, screen_name: &str){
        if self.screens.contains_key(screen_name) {
            return;
        }

        self.screens.insert(screen_name.to_string(), Screen {
            width: Arc::new(Mutex::new(0.0)),
            height: Arc::new(Mutex::new(0.0)),
            image_handle: None, 
            containers: Vec::new(),
            screens: Vec::new()
        });
    }

    #[tokio::main]
    pub async fn add_sprite(&mut self, commands: &mut Commands, scene_uuid: &str, screen_name: &str, position: Vec3, target_width: f32) {
        
        let thread_pool = AsyncComputeTaskPool::get();

        let task_entity = commands.spawn_empty().id();

        let container = commands.spawn(SpatialBundle {
            transform: Transform { translation: position, scale: Vec3::ZERO, ..default() },
            ..default()
        }).id();

        let screen_entity = commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(1.0, 1.0, position.z),
            ..default()
        }).id();

        if let Some(screen) = self.screens.get_mut(screen_name) {
            let width = Arc::clone(&screen.width);
            let height = Arc::clone(&screen.height);
        
            let task = thread_pool.spawn(async move {
                let mut command_queue = CommandQueue::default();

                let start_time = Instant::now();
                let timeout = Duration::from_secs(5);
                let mut scale: f32 = 0.2;
        
                loop {
                    if *width.lock() > 0.0 && *height.lock() > 0.0 {
                        scale = target_width / *width.lock();
                        break;
                    }
                    if start_time.elapsed() > timeout {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }

                command_queue.push(move |world: &mut World| {

                    if let Ok(mut transform) = world.query::<&mut Transform>().get_mut(world, container) {
                        transform.scale = Vec3::new(scale, scale, 1.0);
                    }

                    world
                    .entity_mut(task_entity)
                    .despawn();
                });
                command_queue
            });
            commands.entity(task_entity).insert(ComputeTask((scene_uuid.to_string(), task)));

            screen.containers.push(container);
            screen.screens.push(screen_entity);

            commands.entity(container).add_child(screen_entity);
        }
    }

    pub fn update_screen(&mut self, image_assets: &mut ResMut<Assets<Image>>, screen_name: &str, image: RgbaImage, sprite_query: &mut Query<&mut Handle<Image>>){

        let dynamic_image = Image::from_dynamic(
            DynamicImage::ImageRgba8(image),
            true,
            RenderAssetUsages::default(),
        );
        let width = dynamic_image.width() as f32;
        let height = dynamic_image.height() as f32;
        let image_handle = image_assets.add(dynamic_image);

        self.screens.entry(screen_name.to_string()).and_modify(|e| {

            if let Some(old_handle) = &e.image_handle {
                image_assets.remove(old_handle);
            }
            e.image_handle = Some(image_handle.clone());
            *e.width.lock() = width;
            *e.height.lock() = height;

            if let Some(image_handle) = &e.image_handle {
                for entity in e.screens.iter() {
                    if let Ok(mut screen_handle) = sprite_query.get_mut(*entity) {
                        *screen_handle = image_handle.clone_weak();
                    }
                }   
            }
        });
    }

    pub fn remove_screen(&mut self, commands: &mut Commands, image_assets: &mut ResMut<Assets<Image>>, screen_name: &str){
        if self.screens.contains_key(screen_name) {

            if let Some(screen) = self.screens.get(screen_name) {

                if let Some(old_handle) = &screen.image_handle {
                    image_assets.remove(old_handle);
                }

                for entity in screen.containers.iter() {
                    commands.entity(*entity).despawn_recursive();
                }
            }
            self.screens.remove(screen_name);
        }
    }

    pub fn remove_all(&mut self, commands: &mut Commands){
        for (_, screen) in self.screens.iter() {
            for entity in screen.containers.iter() {
                commands.entity(*entity).despawn_recursive();
            }
        }
        self.screens.clear();
    }
}

impl Default for StreamingResources {
    fn default() -> StreamingResources {
        StreamingResources {
            screens: HashMap::new()
        }
    }
}

#[derive(Resource)]
pub struct NokhwaCamera {
    pub has_opened: bool,
    pub camera_setting: NokhwaCameraSetting,

    pub image_receiver: Option<flume::Receiver<RgbaImage>>,
    pub stop_camera_sender: Option<flume::Sender<bool>>,

}

pub struct NokhwaCameraSetting {
    pub format: RequestedFormat<'static>,
    pub first_camera: CameraInfo
}

impl Default for NokhwaCamera {
    fn default() -> NokhwaCamera {

        NokhwaCamera {
            has_opened: false,
            camera_setting: NokhwaCamera::get_camera_settings(),
            image_receiver: None,

            stop_camera_sender: None,
        }
    }
}

impl NokhwaCamera {
    pub fn get_camera_settings() -> NokhwaCameraSetting{
        let cameras = query(ApiBackend::Auto).unwrap();
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let first_camera = cameras.first().expect("camera not exist");

        let settings = NokhwaCameraSetting {
            format: format,
            first_camera: first_camera.clone()
        };

        return settings;
    }

    pub fn switch_camera(&mut self) {
        if self.has_opened {
            self.has_opened = false;
            if let Some(stop_camera_sender) = &self.stop_camera_sender {
                let _ = stop_camera_sender.send(true);
            }
            if let Some(image_receiver) = &self.image_receiver {
                image_receiver.drain();
            }
            self.image_receiver = None;
            self.stop_camera_sender = None;
            return;
        }
        self.has_opened = true;

        let (image_sender, image_receiver) = bounded::<RgbaImage>(2);
        let (stop_camera_sender, stop_camera_receiver) = bounded::<bool>(1);

        nokhwa_initialize(|_|{});

        let callback = |_| {};

        let mut threaded = CallbackCamera::new(self.camera_setting.first_camera.index().clone(), self.camera_setting.format, callback).unwrap();

        std::thread::spawn(move || {
            threaded.open_stream().unwrap();
            #[allow(clippy::empty_loop)]
            loop {        
                if let Ok(stop) = stop_camera_receiver.try_recv() {
                    if stop {
                        drop(stop_camera_receiver);
                        threaded.stop_stream().unwrap();
                        break;
                    }
                } else {
                    let frame = threaded.poll_frame().unwrap();
                    let new_image = frame.decode_image::<RgbAFormat>().unwrap();
                    let _ = image_sender.send(new_image);
                }
            }
        });

        self.image_receiver = Some(image_receiver);
        self.stop_camera_sender = Some(stop_camera_sender);
    }
}