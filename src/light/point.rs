/// A light that shines in all directions from a specific point (like a lightbulb)
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cast_shadows: bool,
}