// src/volumetrics/fog_config.rs

use glam::Vec3;

/// A localized, 3D bounding box filled with volumetric fog.
/// The fragment shader raymarches through these boxes dynamically!
#[derive(Clone, Debug)]
pub struct FogVolume {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub color: [f32; 3],
    pub density: f32, // The physical thickness of the fog volume
}

impl FogVolume {
    pub fn new(min_bounds: Vec3, max_bounds: Vec3, color: [f32; 3], density: f32) -> Self {
        Self {
            min_bounds,
            max_bounds,
            color,
            density,
        }
    }
}