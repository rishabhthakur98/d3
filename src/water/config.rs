// src/water/config.rs

use glam::Vec3;

/// AAA River configuration using flow dynamics and edge fading.
/// Every variable here is now sent directly to the GPU shaders, allowing you 
/// to tweak the water's look and feel without recompiling shaders.
#[derive(Clone, Debug)]
pub struct RiverConfig {
    // --- Colors ---
    pub deep_color: [f32; 4],
    pub shallow_color: [f32; 4],
    pub foam_color: [f32; 4],
    pub sky_reflection_color: [f32; 3], // The color the water mirrors from the sky
    
    // --- Flow & Waves ---
    pub flow_speed: f32,          // Speed of the water traveling down the Z axis
    pub wave_strength: f32,       // Height of the Gerstner waves
    pub uv_scroll_speed: f32,     // How fast the foam/surface textures move
    
    // --- Visual Shading Tuning ---
    pub specular_exponent: f32,   // High (200.0) = sharp glossy water. Low (32.0) = soft matte water.
    pub fresnel_power: f32,       // Controls mirror effect. 4.0 reflects at sharp angles, 1.0 reflects everywhere.
    pub normal_sample_dist: f32,  // 0.1 = sharp accurate waves. 0.5 = smoothed, averaged lighting.
    pub foam_blend_strength: f32, // 1.0 = thick foam. 0.0 = completely invisible foam.
    
    // --- Geometry ---
    pub length: f32,              // How long the river segment is
    pub width: f32,               // How wide the river is
    pub resolution_length: u32,   // Subdivisions along the flow (Higher = smoother geometry)
    pub resolution_width: u32,    // Subdivisions across the banks
    
    pub position: Vec3,
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            deep_color: [0.02, 0.15, 0.3, 0.95],
            shallow_color: [0.1, 0.4, 0.5, 0.7],
            foam_color: [0.8, 0.9, 0.9, 0.9],
            sky_reflection_color: [0.35, 0.55, 0.85], // Beautiful default sky blue
            
            flow_speed: 2.0,
            wave_strength: 0.25,
            uv_scroll_speed: 0.5,
            
            // Tweaked for a highly realistic, smooth appearance out of the box
            specular_exponent: 64.0, 
            fresnel_power: 4.0,
            normal_sample_dist: 0.5, 
            foam_blend_strength: 0.8,
            
            length: 200.0,
            width: 40.0,
            // Bumped up default resolution to prevent geometric jittering
            resolution_length: 400, 
            resolution_width: 100,
            
            position: Vec3::ZERO,
        }
    }
}