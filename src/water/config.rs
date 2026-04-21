use glam::Vec3;

/// AAA River configuration using flow dynamics and edge fading.
#[derive(Clone, Debug)]
pub struct RiverConfig {
    pub deep_color: [f32; 4],
    pub shallow_color: [f32; 4],
    pub foam_color: [f32; 4],
    
    pub flow_speed: f32,       // Speed of the water traveling down the Z axis
    pub wave_strength: f32,    // Height of the Gerstner waves
    
    pub length: f32,           // How long the river segment is
    pub width: f32,            // How wide the river is
    pub resolution_length: u32,// Subdivisions along the flow
    pub resolution_width: u32, // Subdivisions across the banks
    
    pub position: Vec3,
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            deep_color: [0.02, 0.15, 0.3, 0.95],
            shallow_color: [0.1, 0.4, 0.5, 0.7],
            foam_color: [0.8, 0.9, 0.9, 0.9],
            flow_speed: 2.0,
            wave_strength: 0.25,
            length: 200.0,
            width: 40.0,
            resolution_length: 200,
            resolution_width: 50,
            position: Vec3::ZERO,
        }
    }
}