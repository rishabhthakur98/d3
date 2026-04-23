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

// NEW: Data mapped to the GLSL Fog Volume Raymarcher
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FogVolumeData {
    pub min_bounds: Vec4,    // xyz: Min Bounds, w: padding
    pub max_bounds: Vec4,    // xyz: Max Bounds, w: padding
    pub color_density: Vec4, // xyz: Color, w: Density
}

impl Default for GlobalLightData { fn default() -> Self { Self { direction: Vec4::ZERO, color: Vec4::ZERO } } }
impl Default for SpotLightData { fn default() -> Self { Self { position: Vec4::ZERO, direction: Vec4::ZERO, color: Vec4::ZERO, params: Vec4::ZERO } } }
impl Default for PointLightData { fn default() -> Self { Self { position: Vec4::ZERO, color: Vec4::ZERO } } }
impl Default for FogVolumeData { fn default() -> Self { Self { min_bounds: Vec4::ZERO, max_bounds: Vec4::ZERO, color_density: Vec4::ZERO } } }

/// The main payload sent to the GPU every frame. 
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LightUBO {
    pub ambient_color: Vec4, // xyz: Color, w: Intensity
    pub camera_pos: Vec4,    // xyz: Position, w: Padding
    
    pub global_count: u32,
    pub spot_count: u32,
    pub point_count: u32,          
    pub fog_count: u32,      // Replaced old padding to store fog iteration limit perfectly
    
    pub global_lights: [GlobalLightData; 4],   
    pub spot_lights: [SpotLightData; 100],     
    pub point_lights: [PointLightData; 100],   
    
    // NEW: Support for 10 simultaneous localized fog volumes
    pub fog_volumes: [FogVolumeData; 10],      
}