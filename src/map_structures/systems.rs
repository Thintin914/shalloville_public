use bevy::prelude::*;
use bevy::render::render_resource::FilterMode;
use bevy_entitiles::prelude::{TilemapTexture, TilemapTextureDescriptor};
use bevy_entitiles::render::material::StandardTilemapMaterial;
use bevy_entitiles::tilemap::map::TilemapTextures;

use crate::AnimationTimer;

use super::components::{AnimatedTag, InteractiveType};
use super::resources::MapData;

pub fn load_spritesheet(asset_server: Res<AssetServer>, mut map_data: ResMut<MapData>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, mut materials: ResMut<Assets<StandardTilemapMaterial>>, mut textures: ResMut<Assets<TilemapTextures>>) {
    // tilemap
    let image_handle: Handle<Image> = asset_server.load("atlas/Tilemap.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(map_data.tile_width as u32, map_data.tile_height as u32), (map_data.tile_atlas_width / map_data.tile_width as i32) as u32, (map_data.tile_atlas_height / map_data.tile_height as i32) as u32, None, None);
    let layout_count = layout.len() - 1;
    let layout_handle = texture_atlas_layouts.add(layout);
    map_data.atlas_hashmap.insert("tile".to_string(), (image_handle.clone(), layout_handle, layout_count as i8));

    let tile_material = materials.add(StandardTilemapMaterial::default());
    let tile_texture =  textures.add(TilemapTextures::single(
        TilemapTexture::new(
            image_handle.clone_weak(),
            TilemapTextureDescriptor::new(UVec2 { x: map_data.tile_atlas_width as u32, y: map_data.tile_atlas_height as u32 }, UVec2 { x: map_data.tile_width as u32, y: map_data.tile_height as u32}),
        ),
        FilterMode::Nearest,
    ));

    map_data.tile_material = Some(tile_material);
    map_data.tile_texture = Some(tile_texture);
}

pub fn load_animated_sprite(asset_server: &mut Res<AssetServer>, map_data: &mut ResMut<MapData>, texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>) 
-> Option<(Handle<Image>, Handle<TextureAtlasLayout>, i8)>
{
    let index = map_data.place_animated_index.clone();
    if let Some((image_handle, layout_handle, image_count)) = map_data.atlas_hashmap.get(&format!("anim{}", index)) {
        return Some((image_handle.clone_weak(), layout_handle.clone_weak(), image_count.clone() as i8));
    }
    if let Some(layout) = get_animated_sprite_layout(&index) {
        let image_handle: Handle<Image> = asset_server.load(format!("animated_sprite/{}.png", index));
        let image_count = (layout.len() - 1) as i8;
        let layout_handle = texture_atlas_layouts.add(layout);
        
        let image_handle_clone = image_handle.clone_weak();
        let layout_handle_clone = layout_handle.clone_weak();
    
        map_data.atlas_hashmap.insert(format!("anim{}", index), (image_handle, layout_handle, image_count));
    
        return Some((image_handle_clone, layout_handle_clone, image_count));
    }
    return None;
}

pub fn load_static_sprite(asset_server: &mut Res<AssetServer>, index: String) 
-> Handle<Image>
{
    let image_handle: Handle<Image> = asset_server.load(format!("sprite/{}.png", index));

    return image_handle;
}

pub fn update_animated_sprite(mut sprite_query: Query<(&mut AnimatedTag, &mut TextureAtlas)>, time: Res<Time>, mut local_timer: Local<AnimationTimer>){
    local_timer.collasped_timer += time.delta_seconds();
    if local_timer.collasped_timer > 0.2 {
        local_timer.collasped_timer = 0.0;
        for (mut animated_tag, mut texture_atlas) in sprite_query.iter_mut() {
            animated_tag.current += animated_tag.increment;
            if animated_tag.current > animated_tag.last || animated_tag.current < 0 {
                animated_tag.increment *= -1;
            } else {
                texture_atlas.index = animated_tag.current as usize;
            }
        }
    }
}

fn get_animated_sprite_layout(index: &str) -> Option<TextureAtlasLayout> {
    match index {
        "0" => {Some(TextureAtlasLayout::from_grid(UVec2 { x: 32, y: 32 }, 5, 1, None, None))}
        _ => {None}
    }
}

pub fn get_interactive_trigger(sprite_type: &str, index: i32) ->Vec<(KeyCode, InteractiveType)> {
    let mut interactive: Vec<(KeyCode, InteractiveType)> = Vec::new();
    if sprite_type.eq("static") {
        match index {
            1 => {
                interactive.push((KeyCode::KeyZ, InteractiveType::SwitchScreenShare));
                interactive.push((KeyCode::KeyX, InteractiveType::SwitchCameraShare));
            },
            _ => {}
        }
    } else if sprite_type.eq("animated") {
        match index {
            _ => {}
        }
    }

    return interactive;
}