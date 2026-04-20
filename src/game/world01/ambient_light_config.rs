// src/game/world01/ambient_light_config.rs

use crate::light::global::GlobalLight;
use glam::Vec3;

// Ambient light illuminates the hidden, dark parts of the shadows
pub const AMBIENT_LIGHT_COLOR: [f32; 3] = [1.0, 1.0, 1.0]; 
pub const AMBIENT_LIGHT_INTENSITY: f32 = 0.15; 

/// Returns a list of all suns, moons, or directional fill lights for World 01.
/// You can add, remove, or modify lights in this array to instantly change the sky.
/// Note: The GPU currently supports up to 4 active Global Lights at once.
pub fn get_global_lights() -> Vec<GlobalLight> {
    vec![
        // 1. The Main Sun (Creates the primary warm lighting and casts shadows)
        GlobalLight {
            direction: Vec3::new(1.0, -0.15, 0.5),
            color: [1.0, 0.8, 0.6], // Warm, golden sunset hue
            intensity: 1.5,
            
            // IMPORTANT: Usually, only ONE light should cast shadows to save GPU performance.
            // The renderer will automatically pick the first light with `cast_shadows: true` 
            // to generate the shadow map.
            cast_shadows: true, 
        },
        
        // 2. A Secondary Fill Light / Moon (Adds a cool blue rim-light from the opposite side)
        GlobalLight {
            direction: Vec3::new(-1.0, -0.5, -0.5), // Coming from the opposite direction
            color: [0.4, 0.6, 1.0],                 // Soft, cool blue
            intensity: 0.5,                         // Much weaker than the sun
            cast_shadows: false,                    // No shadows to save performance
        },
        
        // Want a third sun? Just add another GlobalLight block here!
    ]
}