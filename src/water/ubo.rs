// src/water/ubo.rs
use glam::{Vec4, Mat4};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RiverData {
    pub deep_color: Vec4,     
    pub shallow_color: Vec4,  
    pub foam_color: Vec4,     
    pub sky_reflection_color: Vec4, // xyz: SkyColor, w: FoamBlendStrength
    pub params: Vec4,               // x: Time, y: Flow Speed, z: Wave Strength, w: Padding
    pub advanced_params1: Vec4,     // x: SpecularExp, y: FresnelPower, z: NormalSampleDist, w: UVScrollSpeed
}

impl Default for RiverData {
    fn default() -> Self {
        Self {
            deep_color: Vec4::ZERO, shallow_color: Vec4::ZERO, foam_color: Vec4::ZERO,
            sky_reflection_color: Vec4::ZERO, params: Vec4::ZERO, advanced_params1: Vec4::ZERO,
        }
    }
}

/// Upgraded to hold up to 10 dynamically spawned rivers in a single frame
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RiverUBO {
    pub view_proj: Mat4,
    pub camera_pos: Vec4,     
    pub light_dir: Vec4,      
    pub light_color: Vec4,    
    pub river_count: u32,
    pub _pad: [u32; 3],
    pub rivers: [RiverData; 10], 
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RiverPushConstants {
    pub model: Mat4,
    pub river_index: u32,
}