// src/lights/spawnable/spot.rs

use glam::Vec3;

/// A Spot Light emits light in a specific direction, constrained by a cone.
/// Can be spawned dynamically in VirtualCuboids.
#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub position: Vec3, 
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32, 
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
}

impl SpotLight {
    /// Ergonomic builder matching the AssetDefinition style for easy level design
    pub fn new(
        position: Vec3, 
        direction: Vec3, 
        color: [f32; 3], 
        intensity: f32, 
        range: f32, 
        inner_cone_deg: f32, 
        outer_cone_deg: f32, 
        cast_shadows: bool
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color,
            intensity,
            range,
            inner_cone_angle: inner_cone_deg.to_radians(),
            outer_cone_angle: outer_cone_deg.to_radians(),
            cast_shadows,
        }
    }
}