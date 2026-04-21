// src/skybox/ubo.rs

use glam::{Vec4, Mat4};

/// Maps directly to the GLSL SkyboxDisc struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkyboxDiscData {
    pub direction: Vec4, // xyz: Direction, w: Angular Size
    pub color: Vec4,     // xyz: Color, w: Glow Intensity
}

impl Default for SkyboxDiscData {
    fn default() -> Self { Self { direction: Vec4::ZERO, color: Vec4::ZERO } }
}

/// Maps directly to the GLSL SkyboxCrescent struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkyboxCrescentData {
    pub direction: Vec4,     // xyz: Direction, w: Angular Size
    pub color: Vec4,         // xyz: Color, w: Cutout Size
    pub cutout_offset: Vec4, // xyz: Cutout Directional Offset, w: Padding
}

impl Default for SkyboxCrescentData {
    fn default() -> Self { Self { direction: Vec4::ZERO, color: Vec4::ZERO, cutout_offset: Vec4::ZERO } }
}

/// The main payload sent to the GPU for generating the sky procedural colors. Must be 16-byte aligned!
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkyboxUBO {
    pub zenith_color: Vec4,
    pub horizon_color: Vec4,
    pub ground_color: Vec4,
    
    pub disc_count: u32,
    pub crescent_count: u32,
    pub _pad: [u32; 2],
    
    pub discs: [SkyboxDiscData; 5],        // Supports up to 5 Suns/Stars
    pub crescents: [SkyboxCrescentData; 5], // Supports up to 5 Moons
}

/// Push constants reconstruct the view rays from the screen pixels
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkyboxPushConstants {
    pub inv_view_proj: Mat4,
}