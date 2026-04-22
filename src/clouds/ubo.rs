// src/clouds/ubo.rs
use glam::{Vec4, Mat4};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CloudUBO {
    pub inv_view_proj: Mat4,  // Used to reconstruct the view ray from the screen
    pub camera_pos: Vec4,
    pub sun_dir: Vec4,        // xyz: Direction, w: Intensity
    pub sun_color: Vec4,      // xyz: Color, w: Padding
    pub base_color: Vec4,     
    pub highlight_color: Vec4,
    pub params: Vec4,         // x: Time, y: Coverage, z: Density, w: Wind Speed
    pub wind_dir: Vec4,       // xyz: Direction, w: Padding
    pub heights: Vec4,        // x: Height Min, y: Height Max, zw: Padding
}