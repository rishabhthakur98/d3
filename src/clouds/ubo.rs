// src/clouds/ubo.rs
use glam::{Vec4, Mat4};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CloudVolumeData {
    pub min_bounds: Vec4,     // xyz: Min Bounds, w: Coverage
    pub max_bounds: Vec4,     // xyz: Max Bounds, w: Density
    pub base_color: Vec4,     // xyz: Base Color, w: Wind Speed
    pub highlight_color: Vec4,// xyz: Highlight Color, w: Padding
    pub wind_dir: Vec4,       // xyz: Wind Direction, w: Padding
}

impl Default for CloudVolumeData { 
    fn default() -> Self { 
        Self { min_bounds: Vec4::ZERO, max_bounds: Vec4::ZERO, base_color: Vec4::ZERO, highlight_color: Vec4::ZERO, wind_dir: Vec4::ZERO } 
    } 
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CloudUBO {
    pub inv_view_proj: Mat4,  
    pub camera_pos: Vec4,
    pub sun_dir: Vec4,        // xyz: Direction, w: Intensity
    pub sun_color: Vec4,      // xyz: Color, w: Padding
    
    pub time: f32,
    pub cloud_count: u32,
    pub _pad: [u32; 2],
    
    pub clouds: [CloudVolumeData; 10], // Supports up to 10 localized volumetric cloud boxes at once
}