// src/postprocessing/config.rs
#[derive(Clone, Debug)]
pub struct PostProcessConfig {
    pub enabled: bool,
    pub exposure: f32,              // Brightens the scene before tonemapping
    pub gamma: f32,                 // Controls mid-tone brightness (Usually 2.2)
    pub contrast: f32,              // 1.0 = normal, >1.0 = sharp, <1.0 = flat
    pub saturation: f32,            // 1.0 = normal, 0.0 = black & white, >1.0 = vibrant
    pub vignette_strength: f32,     // Darkens the corners of the screen
    pub chromatic_aberration: f32,  // Splits colors at the edges of the lens
    pub film_grain: f32,            // Cinematic noise intensity
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            exposure: 1.2,
            gamma: 2.2,
            contrast: 1.1,
            saturation: 1.15,
            vignette_strength: 1.5,
            chromatic_aberration: 0.8,
            film_grain: 0.05,
        }
    }
}