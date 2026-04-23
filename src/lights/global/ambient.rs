// src/lights/global/ambient.rs

#[derive(Clone, Copy, Debug)]
pub struct AmbientLight {
    pub color: [f32; 3],
    pub intensity: f32,
}