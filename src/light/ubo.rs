// src/light/ubo.rs

/// Matches the GLSL SpotLight struct. Uses Vec4 for strict 16-byte GPU alignment.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpotLightData {
    pub position: glam::Vec4,  // xyz: Position, w: Range
    pub direction: glam::Vec4, // xyz: Direction, w: Intensity
    pub color: glam::Vec4,     // xyz: Color, w: Inner Cone Angle
    pub params: glam::Vec4,    // x: Outer Cone Angle, y: Cast Shadows (1.0 or 0.0), zw: Padding
}

impl Default for SpotLightData {
    fn default() -> Self {
        Self {
            position: glam::Vec4::ZERO,
            direction: glam::Vec4::ZERO,
            color: glam::Vec4::ZERO,
            params: glam::Vec4::ZERO,
        }
    }
}

/// The main payload sent to the GPU every frame containing all lighting info
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LightUBO {
    pub ambient_color: glam::Vec4, // xyz: Color, w: Intensity
    pub global_dir: glam::Vec4,    // xyz: Direction, w: Intensity
    pub global_color: glam::Vec4,  // xyz: Color, w: Padding
    pub camera_pos: glam::Vec4,    // xyz: Position, w: Padding
    
    pub spot_count: u32,
    pub _pad: [u32; 3],            // Padding to reach next 16-byte boundary for the array
    
    pub spot_lights: [SpotLightData; 10], // Supports up to 10 active spotlights at once
}