pub fn from_xy_to_grid(x: f32, y: f32, tile_width: f32, tile_height: f32) -> (f32, f32) {
    let grid_x = (x / tile_width).round();
    let grid_y = (y / tile_height).round();
    (grid_x, grid_y)
}

pub fn from_grid_xy_to_index(grid_x: f32, grid_y: f32, map_col: f32) -> i32 {
    return (grid_y * map_col + grid_x) as i32;
}

pub fn from_index_to_xy(index: i32, tile_width: i32, tile_height: i32, map_col: i32) -> (i32, i32){
    let mut x = 0;
    let mut y = 0;
    if index.ne(&0) && map_col.ne(&0) {
        x = (index % map_col) * tile_width;
        y = (index / map_col) * tile_height;
    }
    return (x, y);
}

pub fn from_index_to_grid_xy(index: i32, map_col: i32) -> (i32, i32){
    let mut x = 0;
    let mut y = 0;
    if index.ne(&0) && map_col.ne(&0) {
        x = index % map_col;
        y = index / map_col;
    }
    return (x, y);
}

pub fn group_numbers(input: String, group_size: usize) -> Result<Vec<Vec<f32>>, String> {
    let numbers: Vec<f32> = input
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| format!("Failed to parse number: {}", s)))
        .collect::<Result<Vec<f32>, String>>()?;

    if numbers.len() % group_size != 0 {
        return Err(format!(
            "Input length ({}) is not divisible by group size ({})",
            numbers.len(),
            group_size
        ));
    }

    Ok(numbers.chunks_exact(group_size).map(|chunk| chunk.to_vec()).collect())
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}