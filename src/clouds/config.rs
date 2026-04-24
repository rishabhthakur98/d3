// src/clouds/config.rs
use glam::Vec3;

// The maximum number of localized cloud bounding boxes allowed on screen.
pub const MAX_CLOUD_VOLUMES: usize = 10;

#[derive(Clone, Debug)]
pub struct CloudVolume {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub base_color: [f32; 3],
    pub highlight_color: [f32; 3],
    pub cloud_coverage: f32,    
    pub cloud_density: f32,     
    pub wind_speed: f32,        
    pub wind_direction: Vec3,   
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