use bevy::prelude::*;


#[derive(Resource)]
pub struct GameResources {
    pub right_bottom_texts: Vec<String>,
    pub chatbar: String,
    pub chat_messages: Vec<String>
}

impl Default for GameResources {
    fn default() -> GameResources {
        GameResources {
            right_bottom_texts: Vec::new(),
            chatbar: String::new(),
            chat_messages: Vec::new()
        }
    }
}