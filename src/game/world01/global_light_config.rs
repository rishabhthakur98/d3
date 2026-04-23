// src/game/world01/global_light_config.rs

use crate::lights::global::directional::GlobalLight;
use glam::Vec3;

/// Returns the persistent global light bodies (Sun, Moon) for the level.
pub fn get_global_lights() -> Vec<GlobalLight> {
    vec![
        // 1. The Main Sun (Casts shadows)
        GlobalLight::new(
            Vec3::new(1.0, -0.15, 0.5), 
            [1.0, 0.8, 0.6], 
            1.5, 
            true
        ),
        
        // 2. Secondary Fill Light / Moon (No shadows)
        GlobalLight::new(
            Vec3::new(-1.0, -0.5, -0.5), 
            [0.4, 0.6, 1.0], 
            0.5, 
            false
        ),
    ]
}