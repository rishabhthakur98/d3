// src/fire/ssbo.rs
use glam::{Vec4, Mat4};
use super::config::MAX_FIRE_PARTICLES;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FireParticleData {
    pub position: Vec4, // xyz: position, w: scale
    pub color: Vec4,    // xyz: color, w: alpha
}

impl Default for FireParticleData {
    fn default() -> Self { Self { position: Vec4::ZERO, color: Vec4::ZERO } }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FireSSBO {
    pub view_proj: Mat4,
    pub camera_right: Vec4, 
    pub camera_up: Vec4,
    pub particle_count: u32,
    pub _pad: [u32; 3], 
    
    // Unbounded in GLSL, bounded in Rust for memory allocation
    pub particles: [FireParticleData; MAX_FIRE_PARTICLES], 
}