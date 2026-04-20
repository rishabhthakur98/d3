// src/light/spot.rs
use glam::Vec3;

/// A light that shines in a specific cone. Now completely independent of meshes!
#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub position: Vec3, // NEW: Position in 3D space
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32, 
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
}