// src/game/world01/controls.rs

use winit::keyboard::KeyCode;
use crate::game::menu::EngineAction;
use super::camera::FreeformCamera;
use super::camera_config;

#[derive(Default)]
pub struct InputState {
    // Positional Movement
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    
    // Rotational Movement (Keyboard Fallbacks)
    pub pitch_up: bool,
    pub pitch_down: bool,
    pub yaw_left: bool,
    pub yaw_right: bool,
    
    // Roll & Reset
    pub roll_cw: bool,
    pub roll_ccw: bool,
    pub reset_view: bool,
}

/// Applies raw mouse delta to the camera Pitch and Yaw
pub fn handle_mouse(camera: &mut FreeformCamera, dx: f64, dy: f64) {
    camera.yaw += (dx as f32) * camera_config::DEFAULT_CAMERA_SENSITIVITY * 100.0;
    camera.pitch -= (dy as f32) * camera_config::DEFAULT_CAMERA_SENSITIVITY * 100.0;

    // Clamp pitch to prevent the camera from flipping upside down natively
    camera.pitch = camera.pitch.clamp(-89.0, 89.0);
    camera.update_camera_vectors();
}

/// Maps keyboard presses to logical state flags
pub fn handle_keyboard(state: &mut InputState, keycode: KeyCode, is_pressed: bool) -> EngineAction {
    match keycode {
        // Translation
        KeyCode::KeyW => state.move_forward = is_pressed,
        KeyCode::KeyS => state.move_backward = is_pressed,
        KeyCode::KeyA => state.move_left = is_pressed,
        KeyCode::KeyD => state.move_right = is_pressed,
        KeyCode::Space => state.move_up = is_pressed,
        KeyCode::ShiftLeft => state.move_down = is_pressed,
        
        // Rotation via Arrows
        KeyCode::ArrowUp => state.pitch_up = is_pressed,
        KeyCode::ArrowDown => state.pitch_down = is_pressed,
        KeyCode::ArrowLeft => state.yaw_left = is_pressed,
        KeyCode::ArrowRight => state.yaw_right = is_pressed,
        
        // Roll & Reset
        KeyCode::KeyE => state.roll_cw = is_pressed,
        KeyCode::KeyQ => state.roll_ccw = is_pressed,
        KeyCode::KeyR => state.reset_view = is_pressed,

        KeyCode::Escape if is_pressed => return EngineAction::ReturnToMenu,
        _ => {}
    }
    
    // FIXED: Changed `EngineAction::None` to `EngineAction::Continue`
    EngineAction::Continue 
}

/// Consumes the input state and updates the camera's physical properties
pub fn update_camera_position(camera: &mut FreeformCamera, state: &InputState, delta_time: f32) {
    let velocity = camera_config::DEFAULT_CAMERA_SPEED * delta_time;
    let turn_speed = camera_config::ARROW_KEY_SENSITIVITY * delta_time;
    let roll_speed = camera_config::ROLL_SENSITIVITY * delta_time;

    let mut moved = false;

    // 1. Handle Orientation Reset
    if state.reset_view {
        camera.reset_orientation();
    } else {
        // 2. Handle Arrow Key Rotation
        if state.pitch_up { camera.pitch += turn_speed; moved = true; }
        if state.pitch_down { camera.pitch -= turn_speed; moved = true; }
        if state.yaw_left { camera.yaw -= turn_speed; moved = true; }
        if state.yaw_right { camera.yaw += turn_speed; moved = true; }
        
        // 3. Handle Roll
        if state.roll_cw { camera.roll += roll_speed; moved = true; }
        if state.roll_ccw { camera.roll -= roll_speed; moved = true; }

        if moved {
            camera.pitch = camera.pitch.clamp(-89.0, 89.0);
            camera.update_camera_vectors();
        }
    }

    // 4. Handle Positional Movement along local axes
    if state.move_forward { camera.position += camera.front * velocity; }
    if state.move_backward { camera.position -= camera.front * velocity; }
    if state.move_left { camera.position -= camera.right * velocity; }
    if state.move_right { camera.position += camera.right * velocity; }
    
    // World Up/Down movement
    if state.move_up { camera.position += glam::Vec3::Y * velocity; }
    if state.move_down { camera.position -= glam::Vec3::Y * velocity; }
}