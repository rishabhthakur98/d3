// src/light/point.rs
use glam::Vec3;

/// A light that shines in all directions from a specific point. Independent of meshes.
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3, // NEW: Position in 3D space
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
}