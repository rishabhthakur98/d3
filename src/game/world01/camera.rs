// src/game/world01/camera.rs

use glam::{Mat4, Vec3};
use super::camera_config::{INITIAL_CAMERA_POS, INITIAL_CAMERA_PITCH, INITIAL_CAMERA_YAW};

pub struct FreeformCamera {
    pub position: Vec3,
    pub pitch: f32, // Up/Down rotation
    pub yaw: f32,   // Left/Right rotation
}

impl Default for FreeformCamera {
    fn default() -> Self {
        Self {
            // Read directly from the clean config file
            position: Vec3::from_array(INITIAL_CAMERA_POS), 
            pitch: INITIAL_CAMERA_PITCH.to_radians(),
            yaw: INITIAL_CAMERA_YAW.to_radians(), 
        }
    }
}

impl FreeformCamera {
    /// Calculates the forward vector based on pitch and yaw
    pub fn get_forward_vector(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Vec3::new(
            cos_yaw * cos_pitch,
            sin_pitch,
            sin_yaw * cos_pitch,
        ).normalize()
    }

    /// Calculates the View Matrix required by the Vertex Shader
    pub fn get_view_matrix(&self) -> Mat4 {
        let forward = self.get_forward_vector();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        Mat4::look_to_rh(self.position, forward, world_up)
    }
}