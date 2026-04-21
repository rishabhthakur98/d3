// src/volumetrics/fog_config.rs

/// Configuration for global Volumetric Fog and God Rays
#[derive(Clone, Debug)]
pub struct FogConfig {
    pub color: [f32; 3],           // The base color of the fog
    pub global_density: f32,       // How thick the fog is over distance
    pub height_falloff: f32,       // How quickly the fog clears up as you go higher
    pub height_offset: f32,        // The baseline Y-axis level of the thickest fog
    pub volumetric_scattering: f32,// The intensity of the God Rays (Light shafts)
}

impl Default for FogConfig {
    fn default() -> Self {
        Self {
            color: [0.6, 0.7, 0.8],     
            global_density: 0.003,      
            height_falloff: 0.05,       
            height_offset: 0.0, 
            volumetric_scattering: 0.08,
        }
    }
}