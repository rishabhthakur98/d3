// src/light/global.rs

use glam::Vec3;

/// A directional light (like the Sun or Moon) that affects everything globally.
/// These are passed directly to the renderer's environment settings, not attached to meshes.
#[derive(Clone, Copy, Debug)]
pub struct GlobalLight {
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
}