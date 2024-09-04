use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct WardrobeResources {
    pub wardrobe_parts: WardrobeParts,

    pub is_name_changable: bool,
    pub is_clothes_changable: bool,

    pub display_comfirm_button: bool,
    pub on_confirm_text: String,
    pub on_confirm_action: OnWardrobeConfirmed,

    pub is_map_changable: bool,
    pub map_total: i32
}

impl Default for WardrobeResources {
    fn default() -> WardrobeResources {
        WardrobeResources {
            wardrobe_parts: WardrobeParts {
                head_total: 2,
                hair_total: 2,
                eyes_total: 2,
                upper_dress_total: 2,
                hip_total: 2,
                legs_total: 3,

                body_parts: BodyParts::default()

            },
            is_name_changable: true,
            is_clothes_changable: true,
            display_comfirm_button: true,
            on_confirm_text: "Enter World".to_string(),
            on_confirm_action: OnWardrobeConfirmed::CreateRoom,
            
            is_map_changable: true,
            map_total: 2
        }
    }
}

pub enum OnWardrobeConfirmed {
    CreateRoom,
    JoinRoom,
    Close
}

pub struct WardrobeParts {
    pub body_parts: BodyParts,

    pub head_total: i8,
    pub hair_total: i8,
    pub eyes_total: i8,
    pub upper_dress_total: i8,
    pub hip_total: i8,
    pub legs_total: i8
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BodyParts {
    pub head: String,
    pub hair: String,
    pub eyes: String,

    pub upper_dress: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub left_hand: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub right_hand: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub body: String,

    pub hip: String,

    pub legs: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub left_leg: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub right_leg: String
}

impl Clone for BodyParts {
    fn clone(&self) -> BodyParts {
        BodyParts {
            head: self.head.to_string(),
            hair: self.hair.to_string(),
            eyes: self.eyes.to_string(),

            upper_dress: self.upper_dress.to_string(),
            left_hand: self.left_hand.to_string(),
            right_hand: self.right_hand.to_string(),
            body: self.body.to_string(),

            hip: self.hip.to_string(),

            legs: self.legs.to_string(),
            left_leg: self.left_leg.to_string(),
            right_leg: self.right_leg.to_string()
        }
    }
}

impl Default for BodyParts {
    fn default() -> BodyParts {
        BodyParts {
            head: "0".to_string(),
            hair: "0".to_string(),
            eyes: "0".to_string(),

            upper_dress: "0".to_string(),
            left_hand: "0".to_string(),
            right_hand: "0".to_string(),
            body: "0".to_string(),

            hip: "0".to_string(),
            
            legs: "0".to_string(),
            left_leg: "0".to_string(),
            right_leg: "0".to_string()
        }
    }
}