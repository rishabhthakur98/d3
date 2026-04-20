// src/light/ubo.rs

/// Matches the GLSL GlobalLight struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GlobalLightData {
    pub direction: glam::Vec4, // xyz: Direction, w: Intensity
    pub color: glam::Vec4,     // xyz: Color, w: Cast Shadows (1.0 or 0.0)
}

/// Matches the GLSL SpotLight struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpotLightData {
    pub position: glam::Vec4,  // xyz: Position, w: Range
    pub direction: glam::Vec4, // xyz: Direction, w: Intensity
    pub color: glam::Vec4,     // xyz: Color, w: Inner Cone Angle
    pub params: glam::Vec4,    // x: Outer Cone Angle, y: Cast Shadows (1.0 or 0.0), zw: Padding
}

/// Matches the GLSL PointLight struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PointLightData {
    pub position: glam::Vec4,  // xyz: Position, w: Range (Radius)
    pub color: glam::Vec4,     // xyz: Color, w: Intensity
}

impl Default for GlobalLightData {
    fn default() -> Self { Self { direction: glam::Vec4::ZERO, color: glam::Vec4::ZERO } }
}

impl Default for SpotLightData {
    fn default() -> Self { Self { position: glam::Vec4::ZERO, direction: glam::Vec4::ZERO, color: glam::Vec4::ZERO, params: glam::Vec4::ZERO } }
}

impl Default for PointLightData {
    fn default() -> Self { Self { position: glam::Vec4::ZERO, color: glam::Vec4::ZERO } }
}

/// The main payload sent to the GPU every frame. Must be 16-byte aligned!
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LightUBO {
    pub ambient_color: glam::Vec4, // xyz: Color, w: Intensity
    pub camera_pos: glam::Vec4,    // xyz: Position, w: Padding
    
    pub global_count: u32,
    pub spot_count: u32,
    pub point_count: u32,          // NEW: Count for Point Lights
    pub _pad: u32,                 // Padding to reach next 16-byte boundary
    
    pub global_lights: [GlobalLightData; 4], // Supports 4 Suns/Moons
    pub spot_lights: [SpotLightData; 10],    // Supports 10 active Spotlights
    pub point_lights: [PointLightData; 10],  // NEW: Supports 10 active Point Lights
}