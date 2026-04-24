// src/smoke/config.rs
use glam::Vec3;

// The strict GPU array limit for billboarding smoke particles.
pub const MAX_SMOKE_PARTICLES: usize = 500;

#[derive(Clone, Debug)]
pub struct SmokeConfig {
    pub position: Vec3,
    pub color: [f32; 3],
    pub spawn_rate: f32,        
    pub particle_lifetime: f32, 
    pub start_scale: f32,       
    pub end_scale: f32,         
    pub rise_speed: f32,        
    pub spread: f32,            
}

impl Default for SmokeConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: [0.6, 0.6, 0.6],
            spawn_rate: 30.0,
            particle_lifetime: 4.0,
            start_scale: 0.5,
            end_scale: 4.0,
            rise_speed: 1.5,
            spread: 1.2,
        }
    }
}