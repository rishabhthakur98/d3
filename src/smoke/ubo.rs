// src/smoke/ubo.rs
use glam::{Vec4, Mat4};
use super::config::MAX_SMOKE_PARTICLES;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ParticleData {
    pub position: Vec4, // xyz: position, w: scale
    pub color: Vec4,    // xyz: color, w: alpha
}

impl Default for ParticleData {
    fn default() -> Self { Self { position: Vec4::ZERO, color: Vec4::ZERO } }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SmokeUBO {
    pub view_proj: Mat4,
    pub camera_right: Vec4, 
    pub camera_up: Vec4,
    pub particle_count: u32,
    pub _pad: [u32; 3], 
    pub particles: [ParticleData; MAX_SMOKE_PARTICLES], 
}