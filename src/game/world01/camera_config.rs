// src/game/world01/camera_config.rs

// --- Movement Settings ---
pub const DEFAULT_CAMERA_SPEED: f32 = 100.0;

// --- Mouse Settings ---
pub const DEFAULT_CAMERA_SENSITIVITY: f32 = 0.002;

// --- Keyboard Orientation Settings ---
// How fast the camera turns when using the arrow keys (Degrees per second)
pub const ARROW_KEY_SENSITIVITY: f32 = 90.0; 

// How fast the camera barrel rolls when holding Q or E (Degrees per second)
pub const ROLL_SENSITIVITY: f32 = 60.0;

// --- Initial Spawn State ---
pub const INITIAL_CAMERA_POS: [f32; 3] = [0.0, 5.0, 0.0];

// Look slightly downwards (-15 degrees) so the ground is visible on screen!
pub const INITIAL_CAMERA_PITCH: f32 = -15.0;
pub const INITIAL_CAMERA_YAW: f32 = -90.0;
pub const INITIAL_CAMERA_ROLL: f32 = 0.0;