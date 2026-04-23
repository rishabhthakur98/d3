// src/lights/spawnable/point.rs

use glam::Vec3;

/// A Point Light radiates outwards in all directions from a specific point in space.
/// Can be spawned dynamically in VirtualCuboids.
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: [f32; 3],
    pub intensity: f32, 
    pub range: f32, 
}

impl PointLight {
    /// Ergonomic builder matching the AssetDefinition style for easy level design
    pub fn new(position: Vec3, color: [f32; 3], intensity: f32, range: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            range,
        }
    }
}