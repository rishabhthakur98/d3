// src/lights/core/ubo.rs

use glam::Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlobalLightData {
    pub direction: Vec4, // xyz: Direction, w: Intensity
    pub color: Vec4,     // xyz: Color, w: Cast Shadows (1.0 or 0.0)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpotLightData {
    pub position: Vec4,  // xyz: Position, w: Range
    pub direction: Vec4, // xyz: Direction, w: Intensity
    pub color: Vec4,     // xyz: Color, w: Inner Cone Angle
    pub params: Vec4,    // x: Outer Cone Angle, y: Cast Shadows (1.0 or 0.0), zw: Padding
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PointLightData {
    pub position: Vec4,  // xyz: Position, w: Range (Radius)
    pub color: Vec4,     // xyz: Color, w: Intensity
}

impl Default for GlobalLightData {
    fn default() -> Self { Self { direction: Vec4::ZERO, color: Vec4::ZERO } }
}

impl Default for SpotLightData {
    fn default() -> Self { Self { position: Vec4::ZERO, direction: Vec4::ZERO, color: Vec4::ZERO, params: Vec4::ZERO } }
}

impl Default for PointLightData {
    fn default() -> Self { Self { position: Vec4::ZERO, color: Vec4::ZERO } }
}

/// EXPANDED: The main payload sent to the GPU every frame. 
/// Expanded arrays to support 100 simultaneous active dynamic lights!
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LightUBO {
    pub ambient_color: Vec4, // xyz: Color, w: Intensity
    pub camera_pos: Vec4,    // xyz: Position, w: Padding
    
    pub global_count: u32,
    pub spot_count: u32,
    pub point_count: u32,          
    pub _pad: u32,           // Padding to reach next 16-byte boundary
    
    pub fog_color: Vec4,     // xyz: Color, w: Global Density
    pub fog_params: Vec4,    // x: Height Falloff, y: Height Offset, z: Volumetric Scattering, w: Padding
    
    pub global_lights: [GlobalLightData; 4],   // Supports 4 Suns/Moons
    pub spot_lights: [SpotLightData; 100],     // SCALED: Supports up to 100 dynamic Spotlights
    pub point_lights: [PointLightData; 100],   // SCALED: Supports up to 100 dynamic Point Lights
}