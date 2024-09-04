use std::sync::Arc;

use aws_sdk_s3::operation::get_object::builders::GetObjectFluentBuilder;
use bevy::tasks::AsyncComputeTaskPool;

use bevy::{
    ecs::world::CommandQueue, prelude::*, render::render_asset::RenderAssetUsages, tasks::{block_on, futures_lite::future::{self}}
};

use crate::editor::resources::Scene;

use super::{components::{ComputeTask, LoadSpriteTask, LoadTilemapTask}, S3Data};

#[tokio::main]
async fn get_bytes(builder: GetObjectFluentBuilder) -> Vec<u8>{
    let mut final_bytes: Vec<u8> = Vec::new();

    if let Ok(mut object) = builder.send().await {
        while let Ok(Some(bytes)) = object.body.try_next().await {
            final_bytes = bytes.to_vec();
        }
    }
    return final_bytes;
}

fn convert_bytes_to_image(bytes: &mut Vec<u8>) -> Option<Image>{
    let mut image: Option<Image> = None;
    if bytes.len() == 0 {
        return None;
    }

    match image::load_from_memory(&bytes) {
        Ok(img) => {
            image = Some(Image::from_dynamic(img, true, RenderAssetUsages::default()));
        },
        Err(e) => {println!("convert_bytes_to_image failed: {:?}", e);}
    }
    return image;
}

#[tokio::main]
pub async fn load_tilemap_from_s3(commands: &mut Commands, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, bucket: String, key: String){
    if let Some(s3_client) = &s3_data.s3_client {
        let full_key = format!("map/{}.txt", key);
        let builder = s3_client.get_object().bucket(bucket.to_string()).key(full_key.to_string());
        let semaphore = s3_data.s3_semaphore.clone();
        let thread_pool = AsyncComputeTaskPool::get();

        let task_entity = commands.spawn_empty().id();

        let arc_tilemap_cached = Arc::clone(&s3_data.tilemap_cached); // Clone the Arc

        let task = thread_pool.spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let mut command_queue = CommandQueue::default();

            if let Some(map_str) = arc_tilemap_cached.read().get(&full_key.to_string()) {
                let cloned_map_str = map_str.to_string();
                command_queue.push(move |world: &mut World| {
                    world
                    .entity_mut(task_entity)
                    .insert(LoadTilemapTask {
                        map_name: key.to_string(),
                        map_str: cloned_map_str
                    })
                    .remove::<ComputeTask>();
                });
                drop(permit);
                return command_queue;
            }

            let bytes = get_bytes(builder);
            let map_str =  match String::from_utf8(bytes.clone()) {
                Ok(text) => text,
                Err(_) => String::new()
            };

            command_queue.push(move |world: &mut World| {
                arc_tilemap_cached.write().insert(full_key.to_string(), map_str.to_string());

                world
                .entity_mut(task_entity)
                .insert(LoadTilemapTask {
                    map_name: key.to_string(),
                    map_str: map_str
                })
                .remove::<ComputeTask>();
            });

            drop(permit);
            command_queue
        });
        commands.entity(task_entity).insert(ComputeTask((scene_uuid.to_string(), task)));
    }
}

#[tokio::main]
pub async fn load_sprite_from_s3(commands: &mut Commands, scene_uuid: &str, s3_data: &mut ResMut<S3Data>, bucket: String, key: String, target: Entity){
    if let Some(s3_client) = &s3_data.s3_client {
        let builder = s3_client.get_object().bucket(bucket.to_string()).key(key.to_string());
        let semaphore = s3_data.s3_semaphore.clone();
        let thread_pool = AsyncComputeTaskPool::get();
        
        let task_entity = commands.spawn_empty().id();

        let arc_image_cached = Arc::clone(&s3_data.image_cached); // Clone the Arc

        let task = thread_pool.spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let mut command_queue = CommandQueue::default();

            if let Some(image_handle) = arc_image_cached.read().get(&key) {
                let ih = image_handle.clone_weak();
                command_queue.push(move |world: &mut World| {
                    world
                    .entity_mut(task_entity)
                    .insert(LoadSpriteTask {
                        image_handle: ih,
                        entity: target
                    })
                    .remove::<ComputeTask>();
                });
                drop(permit);
                return command_queue
            }
            let mut bytes = get_bytes(builder);
            command_queue.push(move |world: &mut World| {
                if let Some(image) = convert_bytes_to_image( &mut bytes) {
                    let image_handle = world.add_asset::<Image>(image);
                    arc_image_cached.write().insert(key.to_string(), image_handle.clone());
                    world
                    .entity_mut(task_entity)
                    .insert(LoadSpriteTask {
                        image_handle: image_handle.clone(),
                        entity: target
                    })
                    .remove::<ComputeTask>();
                } else {
                    world
                    .entity_mut(task_entity)
                    .despawn();
                }
            });
            drop(permit);
            return command_queue
        });
        commands.entity(task_entity).insert(ComputeTask((scene_uuid.to_string(), task)));
    }
}

pub fn execute_tasks(mut commands: Commands, scene: Res<Scene>, mut tasks: Query<(Entity, &mut ComputeTask)>) {
    for (entity, mut task) in &mut tasks {
        let task_scene_uuid = task.0.0.to_string();
        if scene.scene_uuid.ne(&task_scene_uuid) {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0.1)) {
            commands.append(&mut commands_queue);
        }
    }
}