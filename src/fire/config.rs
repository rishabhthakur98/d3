// src/fire/config.rs
use glam::Vec3;

// Extracted Hardcoded GPU array limit to a configurable constant
pub const MAX_FIRE_PARTICLES: usize = 500;

#[derive(Clone, Debug)]
pub struct FireConfig {
    pub position: Vec3,
    pub core_color: [f32; 3],   
    pub edge_color: [f32; 3],   
    pub spawn_rate: f32,        
    pub particle_lifetime: f32, 
    pub start_scale: f32,       
    pub end_scale: f32,         
    pub rise_speed: f32,        
    pub spread: f32,            
}

impl Default for FireConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            core_color: [1.0, 0.8, 0.2], 
            edge_color: [1.0, 0.1, 0.0], 
            spawn_rate: 60.0,            
            particle_lifetime: 0.8,      
            start_scale: 1.5,
            end_scale: 0.1,              
            rise_speed: 4.0,
            spread: 0.8,
        }
    }
}