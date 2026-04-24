// src/vulkan_logic/config/cull.rs

use ash::vk;

/// Intuitive boolean-based configuration for backface culling and winding order.
#[derive(Clone, Copy, Debug)]
pub struct BackfaceCullConfig {
    /// If true, culling is completely disabled (both sides render).
    pub mode_none: bool,
    /// If `mode_none` is false, determines the winding order.
    /// `true` = Clockwise (CW), `false` = Counter-Clockwise (CCW).
    pub mode_cw: bool,
}

impl Default for BackfaceCullConfig {
    fn default() -> Self {
        Self {
            // Default to true to prevent the Y-Axis flip from hiding models
            mode_none: true, 
            mode_cw: true,
        }
    }
}

impl BackfaceCullConfig {
    /// Translates intuitive booleans into direct Vulkan API flags cleanly
    pub fn get_cull_mode(&self) -> vk::CullModeFlags {
        if self.mode_none {
            vk::CullModeFlags::NONE
        } else {
            vk::CullModeFlags::BACK
        }
    }

    /// Translates intuitive booleans into direct Vulkan API front-face constants
    pub fn get_front_face(&self) -> vk::FrontFace {
        if self.mode_cw {
            vk::FrontFace::CLOCKWISE
        } else {
            vk::FrontFace::COUNTER_CLOCKWISE
        }
    }
}