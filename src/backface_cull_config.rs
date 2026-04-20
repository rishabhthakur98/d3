// src/backface_cull_config.rs

// 0 = Clockwise, 1 = Counter-Clockwise, 2 = None
pub const CULL_MODE_CW: u8 = 0;
pub const CULL_MODE_CCW: u8 = 1;
pub const CULL_MODE_NONE: u8 = 2;

// FIXED: Default to NONE to prevent the Y-Axis flip from hiding all buildings
pub const DEFAULT_CULL_MODE: u8 = CULL_MODE_NONE;