use bevy::prelude::*;

#[derive(Resource)]
pub struct MouseData {
    pub grid_x: f32,
    pub grid_y: f32,

    pub window_height: i32,
    pub window_width: i32,

    pub mouse_x: f32,
    pub mouse_y: f32
}


impl Default for MouseData{
    fn default() -> MouseData {
        MouseData {
            grid_x: 0.,
            grid_y: 0.,

            window_height: 0,
            window_width: 0,

            mouse_x: 0.,
            mouse_y: 0.
        }
    }
}