// src/lights/core/config.rs

// --------------------------------------------------------------------------------
// Centralized hardware limits for the lighting and volumetric systems.
// Adjusting these will safely propagate through all Rust buffers and GLSL shaders.
// --------------------------------------------------------------------------------
pub const MAX_GLOBAL_LIGHTS: usize = 4;
pub const MAX_SPOT_LIGHTS: usize = 100;
pub const MAX_POINT_LIGHTS: usize = 100;
pub const MAX_FOG_VOLUMES: usize = 10;