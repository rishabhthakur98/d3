// src/fire/config.rs
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct FireConfig {
    pub position: Vec3,
    pub core_color: [f32; 3],   // The bright center color
    pub edge_color: [f32; 3],   // The fading edge color
    pub spawn_rate: f32,        // Fire spawns rapidly
    pub particle_lifetime: f32, // Fire particles die quickly
    pub start_scale: f32,       // Fire starts large
    pub end_scale: f32,         // Fire shrinks to a tip as it rises
    pub rise_speed: f32,        // Speed of the flames shooting up
    pub spread: f32,            // Width of the bonfire base
}

impl Default for FireConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            core_color: [1.0, 0.8, 0.2], // White/Yellow hot core
            edge_color: [1.0, 0.1, 0.0], // Deep red tips
            spawn_rate: 60.0,            // High spawn rate for dense flames
            particle_lifetime: 0.8,      // Short life (flames disappear fast)
            start_scale: 1.5,
            end_scale: 0.1,              // Shrinks to a point
            rise_speed: 4.0,
            spread: 0.8,
        }
    }
}