// src/game/world01/skybox_config.rs

use crate::skybox::config::{SkyboxConfig, SkyDisc, SkyCrescent};
use glam::Vec3;

/// Defines the specific visual configuration of the Sky for World 01.
pub fn get_skybox_config() -> SkyboxConfig {
    SkyboxConfig {
        // Deep blue transitioning down to a lighter horizon
        zenith_color: [0.05, 0.15, 0.4], 
        horizon_color: [0.5, 0.6, 0.7], 
        ground_color: [0.1, 0.1, 0.1],
        
        discs: vec![
            // Primary warm sun
            SkyDisc {
                direction: Vec3::new(1.0, 0.4, 0.5).normalize(),
                angular_size: 0.005,
                color: [1.0, 0.9, 0.7],
                glow_intensity: 15.0, // High glow bleeds out far into the sky
            },
            // A secondary distant red star
            SkyDisc {
                direction: Vec3::new(-1.0, 0.8, -0.5).normalize(),
                angular_size: 0.002,
                color: [1.0, 0.2, 0.1],
                glow_intensity: 5.0,
            }
        ],
        
        crescents: vec![
            // A huge glowing crescent moon
            SkyCrescent {
                direction: Vec3::new(-0.8, 0.3, 0.5).normalize(),
                angular_size: 0.02,
                color: [0.8, 0.8, 0.9],
                cutout_offset: Vec3::new(0.015, 0.01, 0.0), // Offsets the negative circle cutting the shape
                cutout_size: 0.018,
            }
        ]
    }
}