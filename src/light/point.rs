// src/light/point.rs

use glam::Vec3;

/// A Point Light radiates outwards in all directions from a specific point in space.
/// Think of a glowing magical crystal, a torch, or a bare lightbulb.
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub color: [f32; 3],
    
    /// How bright the core of the light is
    pub intensity: f32, 
    
    /// The radius/range. The light will quadratically fade to zero at this distance.
    pub range: f32, 
}