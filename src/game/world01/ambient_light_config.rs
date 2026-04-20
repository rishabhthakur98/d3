// src/game/world01/ambient_light_config.rs

// Pure configuration for the global ambient light of World 01
// [R, G, B] values from 0.0 to 1.0
pub const AMBIENT_LIGHT_COLOR: [f32; 3] = [1.0, 1.0, 1.0]; 

// How powerful the ambient light is (0.0 is pitch black, 1.0 is fully bright)
// Set to a low 0.1 so the scene is dark, making your streetlights pop!
pub const AMBIENT_LIGHT_INTENSITY: f32 = 0.1;