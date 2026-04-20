use glam::Vec3;

/// A light that shines in a specific cone (like a flashlight or streetlight)
#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32, // For soft edges
    pub outer_cone_angle: f32,
    pub cast_shadows: bool,
}