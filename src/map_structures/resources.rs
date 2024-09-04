use std::collections::HashMap;
use bevy::prelude::*;
use bevy_entitiles::{render::material::StandardTilemapMaterial, tilemap::map::TilemapTextures};
use ::serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct MapData {
    pub map_name: String,
    pub tile_atlas_name: String,

    pub map_row_str: String,
    pub map_row: i32,

    pub map_col_str: String,
    pub map_col: i32,

    pub tile_atlas_width: i32,
    pub tile_atlas_height: i32,

    pub tile_width: f32,
    pub tile_height: f32,

    pub max_y: f32,

    pub is_map_loaded: bool,
    pub spawn_pos: i32,

    pub tile_material: Option<Handle<StandardTilemapMaterial>>,
    pub tile_texture: Option<Handle<TilemapTextures>>,

    pub place_tile_index: String,
    pub place_entity_index: String,
    pub place_animated_index: String,
    pub place_info_index: String,

    pub editor_mode_str: String,
    pub is_info_window_opened: bool,
    pub info_window_title: String,
    pub info_window_content: String,

    pub atlas_hashmap: HashMap<String, (Handle<Image>, Handle<TextureAtlasLayout>, i8)>,
}

impl Default for MapData{
    fn default() -> MapData {
        MapData {
            map_name: "".to_string(),
            tile_atlas_name: "".to_string(),
            
            map_col_str: "0".to_string(),
            map_row: 0,

            map_row_str: "0".to_string(),
            map_col: 0,

            tile_atlas_width: 640,
            tile_atlas_height: 128,

            tile_width: 64.0,
            tile_height: 64.0,

            max_y: 0.0,

            place_tile_index: "0".to_string(),
            place_entity_index: "0".to_string(),
            place_animated_index: "0".to_string(),
            place_info_index: "0".to_string(),

            editor_mode_str: "Free Mode".to_string(),

            is_info_window_opened: false,
            info_window_title: "".to_string(),
            info_window_content: "".to_string(),

            atlas_hashmap: HashMap::new(),

            is_map_loaded: false,
            spawn_pos: 0,

            tile_material: None,
            tile_texture: None
        }
    }

}

#[derive(Resource)]
pub struct MapStructures {
    pub tilemap: Option<Entity>,
    pub tiles: Vec<i32>,
    pub sprite: HashMap<i32, Entity>,
    pub animated_sprite: HashMap<i32, Entity>,
    pub hitbox: HashMap<i32, (bool, Option<Entity>)>,
    pub info: HashMap<String, Vec<(String, String)>>
}

impl Default for MapStructures {
    fn default() -> MapStructures {
        MapStructures {
            tilemap: None,
            tiles: Vec::new(),
            sprite: HashMap::new(),
            animated_sprite: HashMap::new(),
            hitbox: HashMap::new(),
            info: HashMap::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MapSaver {
    pub col: i32,
    pub row: i32,
    pub tilemap: String,
    pub tilesheet: String,
    pub sprites: String,
    pub animated_sprites: String,
    pub hitbox: String,
    pub info: String
}

impl Default for MapSaver {
    fn default() -> MapSaver {
        return MapSaver { col: 0, row: 0, tilemap: String::new(), sprites: String::new(), animated_sprites: String::new(), hitbox: String::new(), tilesheet: String::new(), info: String::new() }
    }
}