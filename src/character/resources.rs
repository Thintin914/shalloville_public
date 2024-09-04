use std::collections::HashMap;
use bevy::prelude::*;
use bevy::prelude::Quat;

#[derive(Resource)]
pub struct CharacterAnimation {
    pub animations: HashMap<String, Track>,
    pub character_offset: HashMap<String, Vec3>,
    pub max: HashMap<String, (i8, TrackLoop)>
}
pub const MAX_FRAME: i8 = 127;

pub struct Track {
    pub hashmap: HashMap<i8, (Vec3, Quat)>
}

pub enum TrackLoop {
    PingPong,
    Restart
}

impl Default for CharacterAnimation{
    fn default() -> CharacterAnimation {
        CharacterAnimation {
            animations: HashMap::from([
                (
                    "idle body anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            ( 0, (Vec3::ZERO, Quat::IDENTITY) ),
                            ( 1, (Vec3 {x: 0.0, y: -0.5, z: 0.0}, Quat::IDENTITY) ),
                            ( 2, (Vec3 {x: 0.0, y: -1.0, z: 0.0}, Quat::IDENTITY) )
                        ])
                    }
                ),
                (
                    "idle left_hand anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            ( 0, (Vec3::ZERO, Quat::IDENTITY) ),
                            ( 1, (Vec3 {x: 0.0, y: -0.5, z: 0.0}, Quat::IDENTITY) ),
                            ( 2, (Vec3 {x: 0.0, y: -1.0, z: 0.0}, Quat::IDENTITY) )
                        ])
                    }
                ),
                (
                    "idle right_hand anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            ( 0, (Vec3::ZERO, Quat::IDENTITY) ),
                            ( 1, (Vec3 {x: 0.0, y: -0.5, z: 0.0}, Quat::IDENTITY) ),
                            ( 2, (Vec3 {x: 0.0, y: -1.0, z: 0.0}, Quat::IDENTITY) )
                        ])
                    }
                ),
                (
                    "walk left_leg anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            (0, (Vec3::ZERO, Quat::IDENTITY)),
                            (1, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (2, (Vec3 {x: 4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.4, 1.0))),
                            (3, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (4, (Vec3::ZERO, Quat::IDENTITY)),
                            (5, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (6, (Vec3 {x: -4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.4, 1.0))),
                            (7, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                        ])
                    }
                ),
                (
                    "walk right_leg anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            (0, (Vec3::ZERO, Quat::IDENTITY)),
                            (1, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (2, (Vec3 {x: -4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.4, 1.0))),
                            (3, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (4, (Vec3::ZERO, Quat::IDENTITY)),
                            (5, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (6, (Vec3 {x: 4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.4, 1.0))),
                            (7, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                        ])
                    }
                ),
                (
                    "walk left_hand anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            (0, (Vec3::ZERO, Quat::IDENTITY)),
                            (1, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (2, (Vec3 {x: -4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.4, 1.0))),
                            (3, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (4, (Vec3::ZERO, Quat::IDENTITY)),
                            (5, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (6, (Vec3 {x: 4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.4, 1.0))),
                            (7, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                        ])
                    }
                ),
                (
                    "walk right_hand anchor".to_string(),
                    Track {
                        hashmap: HashMap::from([
                            (0, (Vec3::ZERO, Quat::IDENTITY)),
                            (1, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (2, (Vec3 {x: 4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.4, 1.0))),
                            (3, (Vec3 {x: 2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, 0.2, 1.0))),
                            (4, (Vec3::ZERO, Quat::IDENTITY)),
                            (5, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                            (6, (Vec3 {x: -4.0, y: 2.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.4, 1.0))),
                            (7, (Vec3 {x: -2.0, y: 0.0, z: 0.0}, Quat::from_xyzw(0.0, 0.0, -0.2, 1.0))),
                        ])
                    }
                ),
            ]),
            character_offset: HashMap::from([
                ("hair".to_string(), Vec3 {x: -4.0, y: 0.0, z: 7.0}),
                ("eyes".to_string(), Vec3 {x: 2.0, y: 0.0, z: 6.0}),
                ("head".to_string(), Vec3 {x: 0.0, y: 17.5, z: 5.0}),
                ("right_hand".to_string(), Vec3 {x: -6.0, y: -1.0, z: 4.0}),
                ("body".to_string(), Vec3 {x: -1.0, y: 12.0, z: 5.0}),
                ("hip".to_string(), Vec3 {x: 0.0, y: 16.0, z: 3.0}),

                ("left_hand".to_string(), Vec3 {x: 8.0, y: -1.0, z: 2.0}),
                ("right_leg".to_string(), Vec3 {x: -3.0, y: -7.0, z: 1.0}),
                ("left_leg".to_string(), Vec3 {x: 4.0, y: -6.0, z: 0.0}),
            ]),
            max: HashMap::from([
                ("idle".to_string(), (3, TrackLoop::PingPong)),
                ("walk".to_string(), (8, TrackLoop::Restart))
            ])
        }
    }
}