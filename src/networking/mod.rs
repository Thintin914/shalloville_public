use bevy::prelude::*;

mod systems;
use systems::*;

pub mod s3;
use s3::S3Plugin;

pub mod rtc;
use rtc::RTCPlugin;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup_aws_clients)
        .add_plugins(S3Plugin)
        .add_plugins(RTCPlugin);

    }
}