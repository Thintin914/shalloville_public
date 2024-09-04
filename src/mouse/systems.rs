use bevy::{prelude::*, window::WindowResized};
use bevy_entitiles::prelude::{TileBuilder, TileLayer, TilemapStorage};

use crate::{camera::components::CameraTag, game::components::Depth, systems::{load_animated_sprite, load_static_sprite}, map_structures::{components::{AnimatedTag, SpriteTag}, resources::{MapData, MapStructures}}, utils::{from_grid_xy_to_index, from_xy_to_grid}, TilemapEditorState};
use super::{components::MouseTag, MouseData};


pub fn init_mouse(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>){
    
    let image_handle = asset_server.load("atlas/Mouse.png");
    let texture_atlas = TextureAtlasLayout::from_grid(UVec2 { x: 64, y: 64 }, 2, 1, None, None);
    let texture_handle = texture_atlases.add(texture_atlas);

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::ZERO,
                ..default()
            },
            texture: image_handle,
            ..default()
        },
        TextureAtlas {
            layout: texture_handle,
            index: 0,
        },
        MouseTag {}
    ));
}

pub fn update_window_size(mut resize_reader: EventReader<WindowResized>, mut mouse_data: ResMut<MouseData>){
    for e in resize_reader.read() {
        mouse_data.window_width = e.width as i32;
        mouse_data.window_height = e.height as i32;
    }
}

pub fn update_mouse_pointer(map_data: Res<MapData>, mut mouse_data: ResMut<MouseData>, mut evr_cursor: EventReader<CursorMoved>, mut mouse_query: Query<(&mut Transform, &mut TextureAtlas), With<MouseTag>>, camera_query: Query<&CameraTag>){
    for ev in evr_cursor.read() {
        if let Ok(camera_tag) = camera_query.get_single(){
            let (x, y) = from_xy_to_grid(ev.position.x - mouse_data.window_width as f32 * 0.5 + camera_tag.cam_x, mouse_data.window_height as f32 - ev.position.y - mouse_data.window_height as f32 * 0.5 + camera_tag.cam_y, map_data.tile_width as f32, map_data.tile_height as f32);
            mouse_data.grid_x = x;
            mouse_data.grid_y = y;
            mouse_data.mouse_x = ev.position.x;
            mouse_data.mouse_y = mouse_data.window_height as f32 - ev.position.y;
            if let Ok((mut transform, mut texture_atlas)) = mouse_query.get_single_mut() {
                let mouse_x = x * map_data.tile_width;
                let mouse_y = y * map_data.tile_height;
                transform.translation = Vec3::new(mouse_x, mouse_y, 2.0);
                match x >= map_data.map_col as f32 || y >= map_data.map_row as f32 || x < 0.0 || y < 0.0 {
                    true => {texture_atlas.index = 1},
                    false => {texture_atlas.index = 0}
                }
            }
        }
    }
}

pub fn onclick_place_tile(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>, mouse_data: Res<MouseData>, map_data: Res<MapData>, mut map_structures: ResMut<MapStructures>, mut tilemap_query: Query<&mut TilemapStorage>){
    
    if buttons.pressed(MouseButton::Left) {
        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }
        let pos = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32) as usize;
        if let Some(tilemap) = map_structures.tilemap {
            if let Ok(mut tilemap_storage) = tilemap_query.get_mut(tilemap) {
                let index = match map_data.place_tile_index.parse::<i32>() {
                    Ok(n) => {n},
                    Err(_) => {0},
                };
                tilemap_storage.set(
                    &mut commands,
                    IVec2 { x: mouse_data.grid_x as i32, y: mouse_data.grid_y as i32 },
                    TileBuilder::new()
                        .with_layer(0, TileLayer::no_flip(index))
                );
                map_structures.tiles[pos] = index;
            }
        }
    }
}

pub fn onclick_place_hitbox(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>, mouse_data: Res<MouseData>, map_data: Res<MapData>, mut tile_storage: ResMut<MapStructures>){
    
    if buttons.pressed(MouseButton::Left) {
        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }
        let x = mouse_data.grid_x * map_data.tile_width;
        let y = mouse_data.grid_y * map_data.tile_height;
        let pos = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32);
        match tile_storage.hitbox.get_mut(&pos) {
            None => {
                if let Some((image_handle, texture_atlas, _count)) = map_data.atlas_hashmap.get("tile") {
                    let hitbox =
                    commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(x, y, 1.0),
                            texture: image_handle.clone_weak(),
                            ..default()
                        },
                        TextureAtlas {
                            layout: texture_atlas.clone_weak(),
                            index: 0
                        }
                    )).id();
                    tile_storage.hitbox.insert(pos, (true, Some(hitbox)));
                }
            },
            Some(_) => {}
        }
    }
}

pub fn onclick_place_entity(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>, mouse_data: Res<MouseData>, mut map_data: ResMut<MapData>, mut asset_server: Res<AssetServer>, mut tilemap_editor_state: ResMut<NextState<TilemapEditorState>>, mut tile_storage: ResMut<MapStructures>){
    
    if buttons.pressed(MouseButton::Left) {
        tilemap_editor_state.set(TilemapEditorState::Free);
        map_data.editor_mode_str = "Free Mode".to_string();
        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }
        
        let x = mouse_data.grid_x * map_data.tile_width;
        let y = mouse_data.grid_y * map_data.tile_height;
        let pos = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32);
        let index = match map_data.place_entity_index.parse::<i8>() {
            Ok(n) => {n},
            Err(_) => {0},
        };
        if let Some(sprite) = tile_storage.sprite.get(&pos) {
            commands.entity(*sprite).despawn();
            tile_storage.sprite.remove(&pos);
        }


        let image_handle = load_static_sprite(&mut asset_server, map_data.place_entity_index.clone());
        let sprite =
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(x, y, 0.0),
                texture: image_handle,
                ..default()
            },
            SpriteTag {
                index: index,
                pos: pos
            },
            Depth {
                index: map_data.max_y - y
            }
        )).id();
        tile_storage.sprite.insert(pos, sprite);
    }
}

pub fn onclick_place_animated_entity(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>, mouse_data: Res<MouseData>, mut map_data: ResMut<MapData>, mut asset_server: Res<AssetServer>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, mut tilemap_editor_state: ResMut<NextState<TilemapEditorState>>, mut tile_storage: ResMut<MapStructures>){
    
    if buttons.pressed(MouseButton::Left) {
        tilemap_editor_state.set(TilemapEditorState::Free);
        map_data.editor_mode_str = "Free Mode".to_string();
        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }

        if let Some((image_handle, layout_handle, image_count)) = load_animated_sprite(&mut asset_server, &mut map_data, &mut texture_atlas_layouts) {
            let x = mouse_data.grid_x * map_data.tile_width;
            let y = mouse_data.grid_y * map_data.tile_height;
            let index = match map_data.place_animated_index.parse::<i8>() {
                Ok(n) => {n},
                Err(_) => {0},
            };
            let pos = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32);

            if let Some(sprite) = tile_storage.animated_sprite.get(&pos) {
                commands.entity(*sprite).despawn();
                tile_storage.animated_sprite.remove(&pos);
            }

            let sprite =
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(x, y, 0.0),
                    texture: image_handle.clone_weak(),
                    ..default()
                },
                TextureAtlas {
                    layout: layout_handle.clone_weak(),
                    index: 0
                },
                AnimatedTag {
                    index: index,
                    pos: pos,
                    current: 0,
                    last: image_count,
                    increment: 1
                },
                Depth {
                    index: map_data.max_y - y
                }
            )).id();
            tile_storage.animated_sprite.insert(pos, sprite);
        }}
        else {
            return;
        }
}

pub fn onclick_place_info(buttons: Res<ButtonInput<MouseButton>>, mouse_data: Res<MouseData>, mut map_data: ResMut<MapData>, mut tilemap_editor_state: ResMut<NextState<TilemapEditorState>>) {
    if buttons.pressed(MouseButton::Left) {
        tilemap_editor_state.set(TilemapEditorState::Free);
        map_data.editor_mode_str = "Free Mode".to_string();

        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }
        let pos = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32);

        map_data.is_info_window_opened = true;
        map_data.place_info_index = pos.to_string();
    }
}

pub fn onpress_remove_entity(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>, map_data: Res<MapData>, mouse_data: Res<MouseData>, mut map_structures: ResMut<MapStructures>){
    if keys.just_pressed(KeyCode::KeyX) {
        if mouse_data.grid_x as i32 >= map_data.map_col || mouse_data.grid_y as i32 >= map_data.map_row || mouse_data.grid_x < 0.0 || mouse_data.grid_y < 0.0 {
            return;
        }
        let index = from_grid_xy_to_index(mouse_data.grid_x, mouse_data.grid_y, map_data.map_col as f32);
        if let Some(sprite) = map_structures.sprite.get(&index) {
            commands.entity(*sprite).despawn_recursive();
            map_structures.sprite.remove(&index);
        }
        if let Some(animated_sprite) = map_structures.animated_sprite.get(&index) {
            commands.entity(*animated_sprite).despawn_recursive();
            map_structures.animated_sprite.remove(&index);
        }
    }
}