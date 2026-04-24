// src/weather/ssbo.rs
use glam::{Vec4, Mat4};
use super::config::MAX_WEATHER_PARTICLES;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WeatherParticleData {
    pub position: Vec4, // xyz: local position, w: random seed
    pub color: Vec4,    // xyz: color, w: alpha
    pub params: Vec4,   // x: Scale, y: Fall Speed, z: Is Rain (1.0 or 0.0), w: Time
}

impl Default for WeatherParticleData {
    fn default() -> Self { 
        Self { position: Vec4::ZERO, color: Vec4::ZERO, params: Vec4::ZERO } 
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WeatherSSBO {
    pub view_proj: Mat4,
    pub camera_pos: Vec4,     
    pub camera_right: Vec4,   
    pub camera_up: Vec4,      
    pub particle_count: u32,
    pub _pad: [u32; 3], 
    // The GPU shader will treat this as an unbounded array `[]`, but Rust 
    // requires a fixed size to allocate the backing memory buffer safely.
    pub particles: [WeatherParticleData; MAX_WEATHER_PARTICLES], 
}