use winit::keyboard::KeyCode;
use glam::Vec3;
use super::camera::FreeformCamera;
use super::camera_config::{DEFAULT_CAMERA_SPEED, DEFAULT_CAMERA_SENSITIVITY};
use crate::game::menu::EngineAction;

pub struct InputState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self { forward: false, backward: false, left: false, right: false, up: false, down: false }
    }
}

/// Handles key presses for camera movement. Escape returns to the menu.
pub fn handle_keyboard(state: &mut InputState, keycode: KeyCode, is_pressed: bool) -> EngineAction {
    match keycode {
        KeyCode::KeyW => state.forward = is_pressed,
        KeyCode::KeyS => state.backward = is_pressed,
        KeyCode::KeyA => state.left = is_pressed,
        KeyCode::KeyD => state.right = is_pressed,
        KeyCode::Space => state.up = is_pressed,
        KeyCode::ShiftLeft => state.down = is_pressed,
        KeyCode::Escape if is_pressed => return EngineAction::ReturnToMenu, // Escape hook!
        _ => {}
    }
    EngineAction::Continue
}

/// Updates the camera's physical position based on current active inputs
pub fn update_camera_position(camera: &mut FreeformCamera, state: &InputState, delta_time: f32) {
    let forward = camera.get_forward_vector();
    let up = Vec3::new(0.0, 1.0, 0.0);
    // Cross product gets the vector pointing to the right
    let right = forward.cross(up).normalize();

    let velocity = DEFAULT_CAMERA_SPEED * delta_time;

    if state.forward { camera.position += forward * velocity; }
    if state.backward { camera.position -= forward * velocity; }
    if state.right { camera.position += right * velocity; }
    if state.left { camera.position -= right * velocity; }
    if state.up { camera.position += up * velocity; }
    if state.down { camera.position -= up * velocity; }
}

/// Handles mouse movement to rotate the camera
pub fn handle_mouse(camera: &mut FreeformCamera, delta_x: f64, delta_y: f64) {
    camera.yaw += (delta_x as f32) * DEFAULT_CAMERA_SENSITIVITY;
    camera.pitch -= (delta_y as f32) * DEFAULT_CAMERA_SENSITIVITY; // Inverted Y axis

    // Clamp pitch to prevent the camera from flipping upside down
    let max_pitch = 89.0_f32.to_radians();
    if camera.pitch > max_pitch { camera.pitch = max_pitch; }
    if camera.pitch < -max_pitch { camera.pitch = -max_pitch; }
}