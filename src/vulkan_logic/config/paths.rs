// src/vulkan_logic/config/paths.rs

// --------------------------------------------------------------------------------
// Centralized configuration for all compiled SPIR-V shader locations.
// --------------------------------------------------------------------------------

pub const SHADER_DIR: &str = "src/vulkan_logic/compiled_shaders/";

// Main Geometry & Shadows
pub const MAIN_VERT: &str = "src/vulkan_logic/compiled_shaders/triangle.vert.spv";
pub const MAIN_FRAG: &str = "src/vulkan_logic/compiled_shaders/triangle.frag.spv";
pub const SHADOW_VERT: &str = "src/vulkan_logic/compiled_shaders/shadow.vert.spv";

// Environment
pub const SKYBOX_VERT: &str = "src/vulkan_logic/compiled_shaders/skybox.vert.spv";
pub const SKYBOX_FRAG: &str = "src/vulkan_logic/compiled_shaders/skybox.frag.spv";
pub const CLOUD_VERT: &str = "src/vulkan_logic/compiled_shaders/clouds.vert.spv";
pub const CLOUD_FRAG: &str = "src/vulkan_logic/compiled_shaders/clouds.frag.spv";

// Particles & VFX
pub const SMOKE_VERT: &str = "src/vulkan_logic/compiled_shaders/smoke.vert.spv";
pub const SMOKE_FRAG: &str = "src/vulkan_logic/compiled_shaders/smoke.frag.spv";
pub const FIRE_VERT: &str = "src/vulkan_logic/compiled_shaders/fire.vert.spv";
pub const FIRE_FRAG: &str = "src/vulkan_logic/compiled_shaders/fire.frag.spv";
pub const WEATHER_VERT: &str = "src/vulkan_logic/compiled_shaders/weather.vert.spv";
pub const WEATHER_FRAG: &str = "src/vulkan_logic/compiled_shaders/weather.frag.spv";

// Water
pub const RIVER_VERT: &str = "src/vulkan_logic/compiled_shaders/river.vert.spv";
pub const RIVER_FRAG: &str = "src/vulkan_logic/compiled_shaders/river.frag.spv";
pub const WATER_VERT: &str = "src/vulkan_logic/compiled_shaders/water.vert.spv";
pub const WATER_FRAG: &str = "src/vulkan_logic/compiled_shaders/water.frag.spv";

// Post Processing
pub const POSTPROCESS_VERT: &str = "src/vulkan_logic/compiled_shaders/postprocess.vert.spv";
pub const POSTPROCESS_FRAG: &str = "src/vulkan_logic/compiled_shaders/postprocess.frag.spv";