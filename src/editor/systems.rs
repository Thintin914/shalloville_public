use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_entitiles::math::TileArea;
use bevy_entitiles::prelude::StandardTilemapBundle;
use bevy_entitiles::prelude::TileBuilder;
use bevy_entitiles::prelude::TileLayer;
use bevy_entitiles::prelude::TileRenderSize;
use bevy_entitiles::prelude::TilemapName;
use bevy_entitiles::prelude::TilemapSlotSize;
use bevy_entitiles::prelude::TilemapStorage;
use bevy_entitiles::prelude::TilemapTransform;
use bevy_entitiles::prelude::TilemapType;
use uuid::Uuid;
use std::collections::HashMap;
use super::components::EditorPreviewImage;
use super::resources::StateName;
use super::Scene;

use std::fs::File;
use std::io::Write;
use std::io::Read;

#[cfg(not(target_arch = "wasm32"))]
use tinyfiledialogs::open_file_dialog;

use crate::character::CharacterExistState;
use crate::character::MovementState;
use crate::game::components::Depth;
use crate::game::MultiplayerRoomState;
use crate::game::TriggerButtonState;
use crate::main_menu::resources::RoomData;
use crate::networking::rtc::RoomMetadataListener;
use crate::networking::s3::components::LoadSpriteTask;
use crate::networking::s3::components::LoadTilemapTask;
use crate::main_menu::MainMenuState;
use crate::nokhwa::StreamingState;
use crate::mouse::resources::MouseData;
use crate::systems::get_interactive_trigger;
use crate::systems::load_animated_sprite;
use crate::systems::load_static_sprite;
use crate::map_structures::components::AnimatedTag;
use crate::map_structures::components::InteractiveTrigger;
use crate::map_structures::components::SpriteTag;
use crate::map_structures::components::MapStructure;
use crate::utils::from_index_to_grid_xy;
use crate::utils::group_numbers;
use crate::wardrobe::WardrobeState;
use crate::{camera::CameraState, map_structures::resources::{MapData, MapSaver, MapStructures}, utils::from_index_to_xy, FocusState, TilemapEditorState};

use super::CreateSceneEvent;
use super::DefaultStates;
use super::EditorState;
use super::TilemapLoadedEvent;

pub fn setup_tilemap_editor(mut commands: Commands, mut create_scene_event: EventWriter<CreateSceneEvent>, mut room_data: ResMut<RoomData>) {
    create_scene(&mut commands, &mut create_scene_event, &mut room_data, "tilemap editor", HashMap::from([
        (StateName::EditorState as u8, 1),
        (StateName::CameraState as u8, 1)
    ]), None);

    commands.spawn((
        SpatialBundle {
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..default()
        },
        EditorPreviewImage {
            is_updated: false,
            image: None
        }
    ));
}

pub fn update_preview_image(mut commands: Commands, mut preview_image_query: Query<(Entity, &mut EditorPreviewImage, &mut Visibility, &mut Transform)>, mut map_data: ResMut<MapData>, mouse_data: Res<MouseData>, mut asset_server: Res<AssetServer>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>) {
    if let Ok((entity, mut preview_image, mut visibility, mut transform)) = preview_image_query.get_single_mut() {
        let x = mouse_data.grid_x * map_data.tile_width;
        let y = mouse_data.grid_y * map_data.tile_height;
        transform.translation = Vec3::new(x, y, 2.0);
        
        if preview_image.is_updated {
            preview_image.is_updated = false;
            if let Some(image) = preview_image.image {
                commands.entity(image).despawn();
                preview_image.image = None;
            }   

            match map_data.editor_mode_str.as_str() {
                "Place Tile Mode" => {
                    if let Some((image_handle, atlas_layout, total)) = map_data.atlas_hashmap.get("tile") {
                        let place_tile_index = match map_data.place_tile_index.parse::<i8>() {
                            Ok(n) => n,
                            Err(_) => 0,
                        };
                        if place_tile_index > *total {
                            return;
                        }
                        *visibility = Visibility::Visible;
                        let tile = commands.spawn( (
                            SpriteBundle {
                                transform: Transform::from_xyz(0.0, 0.0, transform.translation.z),
                                texture: image_handle.clone_weak(),
                                ..default()
                            },
                            TextureAtlas {
                                layout: atlas_layout.clone_weak(),
                                index: place_tile_index as usize
                            }
                        )).id();
                        commands.entity(entity).add_child(tile);
                        preview_image.image = Some(tile);
                    }
                },
                "Place Entity Mode" => {
                    *visibility = Visibility::Visible;
                    let place_entity_index = match map_data.place_entity_index.parse::<i8>() {
                        Ok(n) => n,
                        Err(_) => 0
                    };
                    let image_handle = load_static_sprite(&mut asset_server, place_entity_index.to_string());
                    let sprite =
                    commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(0.0, 0.0, transform.translation.z),
                            texture: image_handle,
                            ..default()
                        },
                        SpriteTag {
                            index: place_entity_index,
                            pos: 0
                        }
                    )).id();
                    commands.entity(entity).add_child(sprite);
                    preview_image.image = Some(sprite);
                },
                "Place Animated Entity Mode" => {
                    *visibility = Visibility::Visible;
                    let place_animated_entity_index = match map_data.place_animated_index.parse::<i8>() {
                        Ok(n) => n,
                        Err(_) => 0
                    };
                    if let Some((image_handle, layout_handle, image_count)) = load_animated_sprite(&mut asset_server, &mut map_data, &mut texture_atlas_layouts) {
                        let sprite =
                        commands.spawn((
                            SpriteBundle {
                                transform: Transform::from_xyz(0.0, 0.0, transform.translation.z),
                                texture: image_handle.clone_weak(),
                                ..default()
                            },
                            TextureAtlas {
                                layout: layout_handle.clone_weak(),
                                index: 0
                            },
                            AnimatedTag {
                                index: place_animated_entity_index,
                                pos: 0,
                                current: 0,
                                last: image_count,
                                increment: 1
                            }
                        )).id();
                        commands.entity(entity).add_child(sprite);
                        preview_image.image = Some(sprite);
                    }
                }
                _ => *visibility = Visibility::Hidden
            }
        }
    }
}

pub fn tilemap_editor(mut commands: Commands, mut contexts: EguiContexts, mut asset_server: Res<AssetServer>, mut create_scene_event: EventWriter<CreateSceneEvent>, mut map_data: ResMut<MapData>, mut room_data: ResMut<RoomData>, mut editor_state: Res<State<EditorState>>, mut map_structures: ResMut<MapStructures>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, mouse_data: ResMut<MouseData>, mut tilemap_editor_state: ResMut<NextState<TilemapEditorState>>, mut camera_state: ResMut<NextState<CameraState>>, mut focus_state: ResMut<NextState<FocusState>>, sprite_query: Query<&SpriteTag>, animated_sprite_query: Query<&AnimatedTag>, mut preview_image_query: Query<&mut EditorPreviewImage>){
    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input() {
        focus_state.set(FocusState::UI);
    } else {
        focus_state.set(FocusState::Game);
    }

    let mut preview_image = preview_image_query.get_single_mut().unwrap();

    egui::Window::new("Tilemap Editor")
    .anchor(egui::Align2::LEFT_TOP, egui::vec2(5.0 ,5.0))
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {
        ui.label("Map Size");
        ui.horizontal(|ui| {
            ui.label("Map Row: ");
            ui.text_edit_singleline(&mut map_data.map_row_str);
        });
        ui.horizontal(|ui| {
            ui.label("Map Col: ");
            ui.text_edit_singleline(&mut map_data.map_col_str);
        });
        if ui.button("Generate").clicked() {
            map_data.map_row_str = match map_data.map_row_str.parse::<i32>() {
                Ok(n) => {
                    if n < 1 {
                        "5".to_string()
                    } else {
                        map_data.map_row_str.clone()
                    }
                }
                Err(_) => {"5".to_string()}
            };
            map_data.map_col_str = match map_data.map_col_str.parse::<i32>() {
                Ok(n) => {
                    if n < 1 {
                        "5".to_string()
                    } else {
                        map_data.map_col_str.clone()
                    }
                }
                Err(_) => {"5".to_string()}
            };

            map_data.map_row = map_data.map_row_str.parse::<i32>().unwrap();
            map_data.map_col = map_data.map_col_str.parse::<i32>().unwrap();
            set_clean_tilemap(&mut commands, &mut create_scene_event,&mut map_data, &mut map_structures, &mut room_data);
        }
        ui.separator();

        ui.add_enabled(false, egui::TextEdit::singleline(&mut map_data.editor_mode_str));

        if ui.button("Free Mode").clicked() && map_data.editor_mode_str.ne("Free Mode") {
            tilemap_editor_state.set(TilemapEditorState::Free);
            camera_state.set(CameraState::Free);
            map_data.editor_mode_str = "Free Mode".to_string();
            preview_image.is_updated = true;
        }

        ui.horizontal(|ui| {
            if ui.button("Place Tile Mode").clicked() && map_data.editor_mode_str.ne("Place Tile Mode") {
                tilemap_editor_state.set(TilemapEditorState::PlaceTile);
                camera_state.set(CameraState::Free);
                map_data.editor_mode_str = "Place Tile Mode".to_string();
                preview_image.is_updated = true;
            }
            if ui.button("-").clicked() {
                let mut index = match map_data.place_tile_index.parse::<i8>() {
                    Ok(n) => {n - 1},
                    Err(_) => {0},
                };
                if let Some((_, _, total)) = map_data.atlas_hashmap.get("tile"){
                    if index < 0 {
                        index = *total;
                    }
                    map_data.place_tile_index = index.to_string();
                    preview_image.is_updated = true;
                }
            }
            ui.text_edit_singleline(&mut map_data.place_tile_index);
            if ui.button("+").clicked() {
                let mut index = match map_data.place_tile_index.parse::<i8>() {
                    Ok(n) => {n + 1},
                    Err(_) => {0},
                };
                if let Some((_, _, total)) = map_data.atlas_hashmap.get("tile"){
                    if index > *total{
                        index = 0;
                    }
                    map_data.place_tile_index = index.to_string();
                    preview_image.is_updated = true;
                }
            }
        });

        if ui.button("Place Hitbox Mode").clicked() {
            tilemap_editor_state.set(TilemapEditorState::PlaceHitbox);
            camera_state.set(CameraState::Free);
            map_data.editor_mode_str = "Place Hitbox Mode".to_string();
            preview_image.is_updated = true;
        }

        ui.horizontal(|ui| {
            if ui.button("Place Entity Mode").clicked() {
                tilemap_editor_state.set(TilemapEditorState::PlaceEntity);
                camera_state.set(CameraState::Free);
                map_data.editor_mode_str = "Place Entity Mode".to_string();
                preview_image.is_updated = true;
            }
            ui.text_edit_singleline(&mut map_data.place_entity_index);
        });

        ui.horizontal(|ui| {
            if ui.button("Place Animated Entity Mode").clicked() {
                tilemap_editor_state.set(TilemapEditorState::PlaceAnimatedEntity);
                camera_state.set(CameraState::Free);
                map_data.editor_mode_str = "Place Animated Entity Mode".to_string();
                preview_image.is_updated = true;
            }
            ui.text_edit_singleline(&mut map_data.place_animated_index);
        });

        ui.horizontal(|ui| {
            if ui.button("Place Info").clicked() {
                tilemap_editor_state.set(TilemapEditorState::PlaceInfo);
                camera_state.set(CameraState::Free);
                map_data.editor_mode_str = "Place Info Mode".to_string();
                preview_image.is_updated = true;
            }
        });

        ui.separator();

        if ui.button("Export").clicked() {

            #[cfg(not(target_arch = "wasm32"))] 
            {
                let mut map_saver = MapSaver {..default()};
                map_saver.col = map_data.map_col;
                map_saver.row = map_data.map_row;

                for counter in 0..map_structures.tiles.len() {
                    let index = map_structures.tiles[counter];
                    map_saver.tilemap.push_str(&format!("{} ", index));
                }
                for (pos, entity) in map_structures.sprite.iter() {
                    let t = sprite_query.get(*entity).unwrap();
                    map_saver.sprites.push_str(&format!("{} {} ", pos, t.index));
                }
                for (pos, entity) in map_structures.animated_sprite.iter() {
                    let t = animated_sprite_query.get(*entity).unwrap();
                    map_saver.animated_sprites.push_str(&format!("{} {} ", pos, t.index));
                }
                for (pos, (is_hitbox, _)) in map_structures.hitbox.iter() {
                    if *is_hitbox {
                        map_saver.hitbox.push_str(&format!("{} ", pos));
                    }
                }
                map_saver.tilesheet = map_data.tile_atlas_name.to_string();
                if let Ok(info) = serde_json::to_string(&map_structures.info) {
                    map_saver.info = info;
                }

                if let Some(path) = dirs::download_dir() {
                    let current_date = chrono::Utc::now();
                    let filename = format!("{}/{}{}.txt", path.display(), current_date.date_naive(), current_date.timestamp());
                    let mut f = File::create(filename).expect("Unable to create file");
                    let content = serde_json::to_string(&map_saver).unwrap();
                    f.write_all(content.as_bytes()).expect("Unable to write data");
                }
            }
        }

        if ui.button("Import").clicked() {
            #[cfg(not(target_arch = "wasm32"))] 
            {
                if let Some(path) = dirs::download_dir() {
                    match open_file_dialog("Open", &format!("{}/", path.display()), None) {
                        Some(file) => {
                            let mut f = File::open(file).expect("Unable to open file");
                            let mut data = String::new();
                            f.read_to_string(&mut data).expect("Unable to read string");
                            let map_saver: MapSaver = serde_json::from_str(&data).unwrap();
                            create_scene(&mut commands, &mut create_scene_event, &mut room_data, "tilemap editor", HashMap::from([
                                (StateName::EditorState as u8, 1)
                            ]), None);

                            map_data.map_name = "editor".to_string();
                            load_tilemap_from_saver(&mut commands, &mut asset_server, map_saver, &mut map_data, &mut room_data, &mut map_structures, &mut editor_state, &mut texture_atlas_layouts);
                        }
                        None => {}
                    }
                }
            }
        }
    });

    egui::Window::new("Cursor")
    .frame(egui::Frame{..default()})
    .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(5.0 ,5.0))
    .resizable(false)
    .title_bar(false)
    .show(ctx, |ui| {

        ui.vertical(|ui| {
            ui.add_enabled(false, egui::TextEdit::singleline(&mut format!("{}, {}", mouse_data.grid_x.to_string(), mouse_data.grid_y.to_string())));
            ui.add_enabled(false, egui::TextEdit::singleline(&mut format!("{}, {}", (mouse_data.grid_x * map_data.tile_width).to_string(), (mouse_data.grid_y * map_data.tile_height).to_string())));
        });

    });

    if map_data.is_info_window_opened {
        egui::Window::new("Tilemap Info")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(5.0 ,5.0))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
    
            ui.add_enabled(false, egui::TextEdit::singleline(&mut map_data.place_info_index));

            ui.text_edit_singleline(&mut map_data.info_window_title);
            ui.text_edit_multiline(&mut map_data.info_window_content);

            if ui.button("Add").clicked() {
                if let Some(info_vec) = map_structures.info.get_mut(&map_data.place_info_index) {
                    info_vec.push((map_data.info_window_title.to_string(), map_data.info_window_content.to_string()));
                } else {
                    map_structures.info.insert(
                        map_data.place_info_index.to_string(),
                        Vec::from([
                            (map_data.info_window_title.to_string(), map_data.info_window_content.to_string())
                        ])
                    );
                }
                map_data.info_window_title.clear();
                map_data.info_window_content.clear();
            }

            ui.separator();

            if let Some(json) = map_structures.info.get(&map_data.place_info_index) {
                for (title, content) in json.iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {}", title, content));
                    });
                }
            }

            ui.separator();

            ui.horizontal(|ui| {    
                if ui.button("Clear All").clicked() {
                    if map_structures.info.contains_key(&map_data.place_info_index) {
                        map_structures.info.remove(&map_data.place_info_index);
                    }
                    map_data.info_window_title.clear();
                    map_data.info_window_content.clear();
                }
    
                if ui.button("Close").clicked() {
                    map_data.info_window_title.clear();
                    map_data.info_window_content.clear();
                    map_data.place_info_index = "0".to_string();
                    map_data.is_info_window_opened = false;
                }
            });

        });
    }
}

fn set_clean_tilemap(commands: &mut Commands, create_scene_event: &mut EventWriter<CreateSceneEvent>, map_data: &mut ResMut<MapData>, map_structures: &mut ResMut<MapStructures>, room_data: &mut ResMut<RoomData>){

    create_scene(commands, create_scene_event, room_data, "tilemap editor", HashMap::from([
        (StateName::EditorState as u8, 1)
    ]), None);

    map_structures.tilemap = None;
    map_structures.tiles.clear();
    map_structures.sprite.clear();
    map_structures.animated_sprite.clear();
    map_structures.hitbox.clear();

    let (_, max_y) = from_index_to_xy(map_data.map_col * map_data.map_row, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
    map_data.max_y = max_y as f32;

    let tilemap_entity = commands.spawn(
(        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, -map_data.max_y - 10.0),
            ..default()
        },
        MapStructure)
    ).id();

    if let Some(tile_material) = &map_data.tile_material {
        if let Some(tile_texture) = &map_data.tile_texture {
            let mut tilemap = StandardTilemapBundle {
                name: TilemapName("tilemap".to_string()),
                transform: TilemapTransform{translation: Vec2::new(map_data.tile_width * -0.5, map_data.tile_height * -0.5), ..default()},
                tile_render_size: TileRenderSize(Vec2 { x: map_data.tile_width, y: map_data.tile_height }),
                slot_size: TilemapSlotSize(Vec2 { x: map_data.tile_width, y: map_data.tile_height }),
                ty: TilemapType::Square,
                storage: TilemapStorage::new(16, tilemap_entity),
                material: tile_material.clone_weak(),
                textures: tile_texture.clone_weak(),
                ..Default::default()
            };
        
            tilemap.storage.fill_rect(
                commands,
                TileArea::new(IVec2::ZERO, UVec2 { x: map_data.map_col as u32, y: map_data.map_row as u32 }),
                TileBuilder::new().with_layer(0, TileLayer::no_flip(1)),
            );
            for _ in 0..map_data.map_col * map_data.map_row {
                map_structures.tiles.push(1);
            }
            commands.entity(tilemap_entity).insert(tilemap);
        }
    }

    map_structures.tilemap = Some(tilemap_entity);
    
    commands.entity(room_data.current_scene.unwrap()).add_child(tilemap_entity);
}

pub fn load_tilemap_from_saver(commands: &mut Commands, mut asset_server: &mut Res<AssetServer>, map_saver: MapSaver, map_data: &mut ResMut<MapData>, room_data:  &mut ResMut<RoomData>, map_structures: &mut ResMut<MapStructures>, editor_state: &mut Res<State<EditorState>>, texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>){
    map_data.is_map_loaded = false;

    map_structures.tilemap = None;
    map_structures.tiles.clear();
    map_structures.sprite.clear();
    map_structures.animated_sprite.clear();
    map_structures.hitbox.clear();
    
    let current_editor_state = editor_state.get();
    
    map_data.tile_atlas_name = map_saver.tilesheet;
    map_data.map_col = map_saver.col;
    map_data.map_col_str = map_saver.col.to_string();
    if !map_saver.info.is_empty() {
        map_structures.info = serde_json::from_str(&map_saver.info).unwrap();
    }

    map_data.map_row = map_saver.row;
    map_data.map_row_str = map_saver.row.to_string();

    let (_, max_y) = from_index_to_xy(map_data.map_col * map_data.map_row, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
    map_data.max_y = max_y as f32;

    let tilemap_entity = commands.spawn(
    (        SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, -map_data.max_y - 10.0),
                ..default()
            },
            MapStructure)
        ).id();

    let mut counter: i32 = 0;
    if let Some(tile_material) = &map_data.tile_material {
        if let Some(tile_texture) = &map_data.tile_texture {
            let mut tilemap = StandardTilemapBundle {
                name: TilemapName("tilemap".to_string()),
                transform: TilemapTransform{translation: Vec2::new(map_data.tile_width * -0.5, map_data.tile_height * -0.5), ..default()},
                tile_render_size: TileRenderSize(Vec2 { x: map_data.tile_width, y: map_data.tile_height }),
                slot_size: TilemapSlotSize(Vec2 { x: map_data.tile_width, y: map_data.tile_height }),
                ty: TilemapType::Square,
                storage: TilemapStorage::new(16, tilemap_entity),
                material: tile_material.clone_weak(),
                textures: tile_texture.clone_weak(),
                ..Default::default()
            };
        
            if let Ok(groups) = group_numbers(map_saver.tilemap, 1) {
                for group in groups.iter(){
                    let index = group[0] as i32;
                    if index.eq(&0) {
                        if current_editor_state.ne(&EditorState::Open) {
                            continue;
                        }
                    }
                    let (x, y) = from_index_to_grid_xy(counter, map_data.map_col);

                    tilemap.storage.set(
                        commands,
                        IVec2 { x, y },
                        TileBuilder::new().with_layer(0, TileLayer::no_flip(index)),
                    );
                    map_structures.tiles.push(index);
                    counter += 1;
                }
            }
            commands.entity(tilemap_entity).insert(tilemap);
        }
    }

    map_structures.tilemap = Some(tilemap_entity);

    let sprite_entity = commands.spawn(
(        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MapStructure)
    ).id();


    if let Ok(groups) = group_numbers(map_saver.sprites, 2) {
        for group in groups.iter(){
            let pos = group[0] as i32;
            let index = group[1] as i32;
            let (x, y) = from_index_to_xy(pos, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);

            if index.eq(&0) {
                map_data.spawn_pos = pos;
                if current_editor_state.ne(&EditorState::Open) {
                    continue;
                }
            }

            let image_handle = load_static_sprite(&mut asset_server, index.to_string());
            let sprite =
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                    texture: image_handle,
                    ..default()
                },
                SpriteTag {
                    index: index as i8,
                    pos: pos
                },
                Depth {
                    index: (max_y - y) as f32 + map_data.tile_height * 0.4
                }
            )).id();

            let interactive = get_interactive_trigger("static", index);
            if interactive.len() > 0 {
                commands.entity(sprite).insert(InteractiveTrigger {trigger: interactive});
            }
            map_structures.sprite.insert(pos, sprite);
            commands.entity(sprite_entity).add_child(sprite);
        }
    }

    if let Ok(groups) = group_numbers(map_saver.animated_sprites, 2) {
        for group in groups.iter(){
            let pos = group[0] as i32;
            let index = group[1] as i32;

            if let Some((image_handle, layout_handle, image_count)) = load_animated_sprite(&mut asset_server, map_data, texture_atlas_layouts) {
                let (x, y) = from_index_to_xy(pos, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
                let sprite =
                commands.spawn((
                    SpriteBundle {
                        transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                        texture: image_handle.clone_weak(),
                        ..default()
                    },
                    TextureAtlas {
                        layout: layout_handle.clone_weak(),
                        index: 0
                    },
                    AnimatedTag {
                        index: index as i8,
                        pos: pos,
                        current: 0,
                        last: image_count,
                        increment: 1
                    },
                    Depth {
                        index: (max_y - y) as f32 + map_data.tile_height * 0.4
                    }
                )).id();

                let interactive = get_interactive_trigger("animated", index);
                if interactive.len() > 0 {
                    commands.entity(sprite).insert(InteractiveTrigger {trigger: interactive});
                }
                map_structures.animated_sprite.insert(pos, sprite);
                commands.entity(sprite_entity).add_child(sprite);
            }
        }
    }

    if let Ok(groups) = group_numbers(map_saver.hitbox, 1) {
        for group in groups.iter(){
            let pos = group[0] as i32;
            let mut hitbox: Option<Entity> = None;
            if current_editor_state.eq(&EditorState::Open) {
                if let Some((image_handle, atlas_layout_handle, _)) = map_data.atlas_hashmap.get("tile") {
                    let (x, y) = from_index_to_xy(pos, map_data.tile_width as i32, map_data.tile_height as i32, map_data.map_col);
                    hitbox = 
                    Some(commands.spawn((
                        SpriteBundle {
                            transform: Transform::from_xyz(x as f32, y as f32, 5.0),
                            texture: image_handle.clone_weak(),
                            ..default()
                        },
                        TextureAtlas {
                            layout: atlas_layout_handle.clone_weak(),
                            index: 0,
                        }
                    )).id());
                    commands.entity(sprite_entity).add_child(hitbox.unwrap());
                }
            }
            map_structures.hitbox.insert(pos, (true, hitbox));
        }
    }

    if let Some(scene) = room_data.current_scene {
        commands.entity(scene).add_child(tilemap_entity);
        commands.entity(scene).add_child(sprite_entity);
    }
    map_data.is_map_loaded = true;
}

pub fn load_tilemap_event_listener(events_query: Query<(Entity, &LoadTilemapTask)>, mut commands: Commands, mut asset_server: Res<AssetServer>, mut map_data: ResMut<MapData>, mut room_data: ResMut<RoomData>, mut editor_state: Res<State<EditorState>>, mut tile_storage: ResMut<MapStructures>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, mut tilemap_loaded_event: EventWriter<TilemapLoadedEvent>, map_structures_query: Query<(Entity, &MapStructure)>, mut streaming_state: ResMut<NextState<StreamingState>>){
    for (entity, load_tilemap_task) in events_query.iter() {
        if load_tilemap_task.map_str.is_empty() {
            commands.entity(entity).despawn();
            map_data.is_map_loaded = true;
            continue;
        }
        for (entity, _) in map_structures_query.iter(){
            commands.entity(entity).despawn_recursive();
        }

        map_data.map_name = load_tilemap_task.map_name.to_string();
        if let Ok(map_saver) = serde_json::from_str::<MapSaver>(&load_tilemap_task.map_str) {
            streaming_state.set(StreamingState::Close);
            load_tilemap_from_saver(&mut commands, &mut asset_server, map_saver, &mut map_data, &mut room_data, &mut tile_storage, &mut editor_state, &mut texture_atlas_layouts);
            tilemap_loaded_event.send(TilemapLoadedEvent());
            commands.entity(entity).despawn();
        }
    }
}

pub fn load_sprite_event_listener(events_query: Query<(Entity, &LoadSpriteTask)>, mut commands: Commands, mut sprite_query: Query<&mut Handle<Image>>){
    for (entity, load_sprite_task) in events_query.iter() {
        if let Ok(mut sprite_handle) = sprite_query.get_mut(load_sprite_task.entity) {
            *sprite_handle = load_sprite_task.image_handle.clone();
        }
        commands.entity(entity).despawn();
    }
}

pub fn create_scene(commands: &mut Commands, create_scene_event: &mut EventWriter<CreateSceneEvent>, room_data:  &mut ResMut<RoomData>, scene_name: &str, states: HashMap<u8, u8>, dynamic_states: Option<Vec<u8>>) -> Entity {
    if let Some(scene) = room_data.current_scene {
        commands.entity(scene).despawn_recursive();
    }
    
    let scene = commands.spawn(
        SpatialBundle {
            ..default()
        }
    ).id();

    let mut additional_states: Vec<u8> = Vec::new();
    if let Some(states) = dynamic_states {
        additional_states = states;
    }
    create_scene_event.send(CreateSceneEvent((scene_name.to_string(), states, additional_states)));
    room_data.current_scene = Some(scene);
    return scene;
}

pub fn on_create_scene(mut create_scene_event: EventReader<CreateSceneEvent>, mut default_states: ResMut<DefaultStates>, mut scene: ResMut<Scene>, mut main_menu_state: ResMut<NextState<MainMenuState>>, mut wardrobe_state: ResMut<NextState<WardrobeState>>, mut streaming_state: ResMut<NextState<StreamingState>>, mut trigger_button_state: ResMut<NextState<TriggerButtonState>>, mut camera_state: ResMut<NextState<CameraState>>, mut movement_state: ResMut<NextState<MovementState>>, mut character_exist_state: ResMut<NextState<CharacterExistState>>, mut multiplayer_room_state: ResMut<NextState<MultiplayerRoomState>>, mut editor_state: ResMut<NextState<EditorState>>, mut room_metadata_listener_state: ResMut<NextState<RoomMetadataListener>>) {
    for ev in create_scene_event.read() {

        scene.scene_name = ev.0.0.to_string();
        scene.scene_uuid = uuid::Uuid::new_v4().to_string();

        let mut reset_dict: Vec<u8> = default_states.prev.clone();
        reset_dict.extend(default_states.dynamic.clone());
        reset_dict.retain(|&state_name| !ev.0.1.contains_key(&state_name));

        let mut new_states: HashMap<u8, u8> = HashMap::new();
        for state_name in reset_dict.iter() {
            let state_name_cloned = state_name.clone();
            if let Some(default_state) = default_states.hashmap.get(&state_name_cloned) {
                new_states.insert(state_name_cloned, default_state.clone());
            }
        }
        default_states.prev.clear();
        for (state_name, enum_index) in ev.0.1.iter() {
            let state_name_cloned = state_name.clone();
            new_states.insert(state_name_cloned, *enum_index);
            default_states.prev.push(state_name_cloned);
        }

        for (state_name, enum_index) in new_states.iter() {
            let state_index = *state_name;
            let index = *enum_index;
            match StateName::try_from(state_index).unwrap() {
                StateName::MainMenuState => main_menu_state.set(MainMenuState::try_from(index).unwrap()),
                StateName::WardrobeState => wardrobe_state.set(WardrobeState::try_from(index).unwrap()),
                StateName::StreamingState => streaming_state.set(StreamingState::try_from(index).unwrap()),
                StateName::TriggerButtonState => trigger_button_state.set(TriggerButtonState::try_from(index).unwrap()),
                StateName::CameraState => camera_state.set(CameraState::try_from(index).unwrap()),
                StateName::MovementState => movement_state.set(MovementState::try_from(index).unwrap()),
                StateName::CharacterExistState => character_exist_state.set(CharacterExistState::try_from(index).unwrap()),
                StateName::MultiplayerRoomState => multiplayer_room_state.set(MultiplayerRoomState::try_from(index).unwrap()),
                StateName::EditorState => editor_state.set(EditorState::try_from(index).unwrap()),
                StateName::RoomMetadataListener => room_metadata_listener_state.set(RoomMetadataListener::try_from(index).unwrap())
            }
        }
        default_states.dynamic = ev.0.2.clone();
    }
}