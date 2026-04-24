// src/water/config.rs
use glam::Vec3;

// The maximum number of dynamic rivers allowed to be sent to the GPU in a single draw batch.
pub const MAX_RIVERS: usize = 10;

#[derive(Clone, Debug)]
pub struct RiverConfig {
    pub position: Vec3,
    pub orientation: Vec3, 
    pub scale: Vec3,       
    
    pub deep_color: [f32; 4],
    pub shallow_color: [f32; 4],
    pub foam_color: [f32; 4],
    pub sky_reflection_color: [f32; 3], 
    
    pub flow_speed: f32,          
    pub wave_strength: f32,       
    pub uv_scroll_speed: f32,     
    
    pub specular_exponent: f32,   
    pub fresnel_power: f32,       
    pub normal_sample_dist: f32,  
    pub foam_blend_strength: f32, 
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Vec3::ZERO,
            scale: Vec3::new(40.0, 1.0, 200.0), 

            deep_color: [0.02, 0.15, 0.3, 0.95],
            shallow_color: [0.1, 0.4, 0.5, 0.7],
            foam_color: [0.8, 0.9, 0.9, 0.9],
            sky_reflection_color: [0.35, 0.55, 0.85], 
            
            flow_speed: 2.0,
            wave_strength: 0.25,
            uv_scroll_speed: 0.5,
            
            specular_exponent: 64.0, 
            fresnel_power: 4.0,
            normal_sample_dist: 0.5, 
            foam_blend_strength: 0.8,
        }
    }
}