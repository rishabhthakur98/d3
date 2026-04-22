// src/weather/ubo.rs
use glam::{Vec4, Mat4};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WeatherParticleData {
    pub position: Vec4, // xyz: local position, w: random seed
}

impl Default for WeatherParticleData {
    fn default() -> Self { Self { position: Vec4::ZERO } }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WeatherUBO {
    pub view_proj: Mat4,
    pub camera_pos: Vec4,     
    pub camera_right: Vec4,   
    pub camera_up: Vec4,      
    pub color: Vec4,          
    pub params: Vec4,         // x: Scale, y: Fall Speed, z: Is Rain (1.0 or 0.0), w: Time
    pub wind: Vec4,           // xyz: Wind Direction
    pub particle_count: u32,
    pub _pad: [u32; 3], 
    pub particles: [WeatherParticleData; 3000], // Massive swarm sent to GPU
}