// src/skybox/config.rs

use glam::Vec3;

/// The main generic configuration for a skybox. 
/// You can return this from any specific World's config file.
#[derive(Clone, Debug)]
pub struct SkyboxConfig {
    pub zenith_color: [f32; 3],  // The color straight up in the sky
    pub horizon_color: [f32; 3], // The color at the horizon line
    pub ground_color: [f32; 3],  // The color below the horizon line
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

/// Represents a generic Sun, Star, or celestial disc.
#[derive(Clone, Debug)]
pub struct SkyDisc {
    pub direction: Vec3,
    pub angular_size: f32, // Size of the disc in the sky
    pub color: [f32; 3],
    pub glow_intensity: f32, // Soft edge bleed outward
}

/// Represents a Moon or similar celestial body with a cutout.
#[derive(Clone, Debug)]
pub struct SkyCrescent {
    pub direction: Vec3,
    pub angular_size: f32,
    pub color: [f32; 3],
    pub cutout_offset: Vec3, // Directional offset of the invisible cutout circle
    pub cutout_size: f32,    // Size of the invisible cutout circle
}