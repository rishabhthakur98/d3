// src/light/spot.rs

use glam::Vec3;

/// A Spot Light emits light in a specific direction, constrained by a cone.
/// Think of a flashlight or a streetlight.
#[derive(Clone, Copy, Debug)]
pub struct SpotLight {
    pub position: Vec3, 
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    
    /// Angle (in radians) where the light is at 100% brightness
    pub inner_cone_angle: f32, 
    
    /// Angle (in radians) where the light fades completely to 0%
    pub outer_cone_angle: f32,
    
    /// Determines if this specific light should generate a shadow map
    pub cast_shadows: bool,
}