// src/game/world01/ambient_light_config.rs

// Ambient light illuminates the hidden, dark parts of the shadows
pub const AMBIENT_LIGHT_COLOR: [f32; 3] = [1.0, 1.0, 1.0]; 
pub const AMBIENT_LIGHT_INTENSITY: f32 = 0.15; 

// Global Light (Sun) Configuration
pub const SUN_DIRECTION: [f32; 3] = [1.0, -0.15, 0.5]; 
pub const SUN_COLOR: [f32; 3] = [1.0, 0.8, 0.6]; 
pub const SUN_INTENSITY: f32 = 1.5;