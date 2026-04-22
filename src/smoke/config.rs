// src/smoke/config.rs
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct SmokeConfig {
    pub position: Vec3,
    pub color: [f32; 3],
    pub spawn_rate: f32,        // How many particles spawn per second
    pub particle_lifetime: f32, // How long they live before vanishing
    pub start_scale: f32,       // Size when spawned
    pub end_scale: f32,         // Size when they vanish (smoke expands!)
    pub rise_speed: f32,        // How fast it floats up
    pub spread: f32,            // How wide the smoke plume gets
}

impl Default for SmokeConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: [0.6, 0.6, 0.6],
            spawn_rate: 30.0,
            particle_lifetime: 4.0,
            start_scale: 0.5,
            end_scale: 4.0,
            rise_speed: 1.5,
            spread: 1.2,
        }
    }
}