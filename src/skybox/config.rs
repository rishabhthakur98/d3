// src/skybox/config.rs
use glam::Vec3;

// Maximum procedural suns and moons supported.
pub const MAX_SKYBOX_DISCS: usize = 5;
pub const MAX_SKYBOX_CRESCENTS: usize = 5;

#[derive(Clone, Debug)]
pub struct SkyboxConfig {
    pub zenith_color: [f32; 3],  
    pub horizon_color: [f32; 3], 
    pub ground_color: [f32; 3],  
    pub discs: Vec<SkyDisc>,
    pub crescents: Vec<SkyCrescent>,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            zenith_color: [0.0, 0.0, 0.0],
            horizon_color: [0.0, 0.0, 0.0],
            ground_color: [0.0, 0.0, 0.0],
            discs: Vec::new(),
            crescents: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkyDisc {
    pub direction: Vec3,
    pub angular_size: f32, 
    pub color: [f32; 3],
    pub glow_intensity: f32, 
}

#[derive(Clone, Debug)]
pub struct SkyCrescent {
    pub direction: Vec3,
    pub angular_size: f32,
    pub color: [f32; 3],
    pub cutout_offset: Vec3, 
    pub cutout_size: f32,    
}