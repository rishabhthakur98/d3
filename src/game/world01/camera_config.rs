// src/game/world01/camera_config.rs

pub const DEFAULT_CAMERA_SPEED: f32 = 10.0;
pub const DEFAULT_CAMERA_SENSITIVITY: f32 = 0.002;

pub const INITIAL_CAMERA_POS: [f32; 3] = [0.0, 5.0, 0.0];

// FIXED: Look slightly downwards (-15 degrees) so the ground is visible on screen!
pub const INITIAL_CAMERA_PITCH: f32 = -15.0;
pub const INITIAL_CAMERA_YAW: f32 = -90.0;