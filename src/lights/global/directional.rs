// src/lights/global/directional.rs

use glam::Vec3;

/// A directional light (like the Sun or Moon) that affects everything globally.
/// Loaded continuously via global atmospheric configs.
#[derive(Clone, Copy, Debug)]
pub struct GlobalLight {
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
}

impl GlobalLight {
    pub fn new(direction: Vec3, color: [f32; 3], intensity: f32, cast_shadows: bool) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
            cast_shadows,
        }
    }
}