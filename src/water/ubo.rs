use glam::{Vec4, Mat4};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RiverUBO {
    pub view_proj: Mat4,
    pub camera_pos: Vec4,     
    pub light_dir: Vec4,      
    pub light_color: Vec4,    
    pub deep_color: Vec4,     
    pub shallow_color: Vec4,  
    pub foam_color: Vec4,     
    pub params: Vec4, // x: Time, y: Flow Speed, z: Wave Strength, w: Padding
}

impl Default for RiverUBO {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY, camera_pos: Vec4::ZERO, light_dir: Vec4::ZERO,
            light_color: Vec4::ZERO, deep_color: Vec4::ZERO, shallow_color: Vec4::ZERO,
            foam_color: Vec4::ZERO, params: Vec4::ZERO,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RiverPushConstants {
    pub model: Mat4,
}