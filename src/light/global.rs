use glam::Vec3;

/// A directional light (like the Sun) that affects everything globally
#[derive(Clone, Copy, Debug)]
pub struct GlobalLight {
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub cast_shadows: bool,
}