use bevy::prelude::*;

use aws_config::load_from_env;
use aws_sdk_s3 as s3;

use super::s3::resources::S3Data;

#[tokio::main]
pub async fn setup_aws_clients(mut s3_data: ResMut<S3Data>){
    let myconfig = load_from_env().await;

    let s3_client = s3::Client::new(&myconfig);
    s3_data.s3_client = Some(s3_client);

}