// src/postprocessing/ssbo.rs
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PostProcessSSBO {
    pub enabled: f32,
    pub exposure: f32,
    pub gamma: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub vignette_strength: f32,
    pub chromatic_aberration: f32,
    pub film_grain: f32,
    pub time: f32,
    pub _pad: [f32; 3], // 16-byte alignment
}