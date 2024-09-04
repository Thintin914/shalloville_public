use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use bevy::prelude::*;
use aws_sdk_s3 as s3;
use tokio::sync::Semaphore;

#[derive(Resource)]
pub struct S3Data {
    pub s3_client: Option<s3::Client>,
    pub s3_semaphore: Arc<Semaphore>,
    pub tilemap_cached: Arc<RwLock<HashMap<String, String>>>,
    pub image_cached: Arc<RwLock<HashMap<String, Handle<Image>>>>,
}

impl Default for S3Data{
    fn default() -> S3Data {
        S3Data {
            s3_client: None,
            s3_semaphore: Arc::new(Semaphore::new(1)),
            tilemap_cached: Arc::new(RwLock::new(HashMap::new())),
            image_cached: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}