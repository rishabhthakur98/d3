// src/clouds/config.rs
use glam::Vec3;

/// A localized, 3D bounding box filled with volumetric clouds.
/// Completely decoupled from global space so you can have low foggy clouds over a swamp
/// and high fluffy clouds over a mountain in the same map!
#[derive(Clone, Debug)]
pub struct CloudVolume {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub base_color: [f32; 3],
    pub highlight_color: [f32; 3],
    pub cloud_coverage: f32,    // 0.0 = clear sky, 1.0 = overcast
    pub cloud_density: f32,     // How thick/dark the clouds are
    pub wind_speed: f32,        // Speed of clouds moving across the sky
    pub wind_direction: Vec3,   // Which way the wind blows
}

impl CloudVolume {
    pub fn new(min_bounds: Vec3, max_bounds: Vec3) -> Self {
        Self {
            min_bounds,
            max_bounds,
            base_color: [0.5, 0.6, 0.7],       
            highlight_color: [1.0, 1.0, 1.0],  
            cloud_coverage: 0.45,
            cloud_density: 0.8,
            wind_speed: 15.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.5).normalize(),
        }
    }
}