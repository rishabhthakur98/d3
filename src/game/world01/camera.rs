// src/game/world01/camera.rs

use glam::{Mat4, Vec3, Quat};
use super::camera_config;

pub struct FreeformCamera {
    pub position: Vec3,
    
    // Euler angles in degrees
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,

    // Local spatial vectors
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
}

impl Default for FreeformCamera {
    fn default() -> Self {
        let mut cam = Self {
            position: Vec3::from_array(camera_config::INITIAL_CAMERA_POS),
            pitch: camera_config::INITIAL_CAMERA_PITCH,
            yaw: camera_config::INITIAL_CAMERA_YAW,
            roll: camera_config::INITIAL_CAMERA_ROLL,
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::Y,
            right: Vec3::X,
        };
        cam.update_camera_vectors();
        cam
    }
}

impl FreeformCamera {
    /// Resets the rotation of the camera back to spawn values, but keeps the current position intact
    pub fn reset_orientation(&mut self) {
        self.pitch = camera_config::INITIAL_CAMERA_PITCH;
        self.yaw = camera_config::INITIAL_CAMERA_YAW;
        self.roll = camera_config::INITIAL_CAMERA_ROLL;
        self.update_camera_vectors();
    }

    /// Recalculates the Front, Right, and Up vectors based on Pitch, Yaw, and Roll
    pub fn update_camera_vectors(&mut self) {
        // 1. Calculate standard Forward direction from Pitch and Yaw
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();

        self.front = Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();

        // 2. Calculate baseline Right and Up without any roll
        let world_up = Vec3::Y;
        let right_base = self.front.cross(world_up).normalize();
        let up_base = right_base.cross(self.front).normalize();

        // 3. Apply the Roll! 
        // We do this by rotating the base Right and Up vectors around the Front vector.
        let roll_rad = self.roll.to_radians();
        let roll_rotation = Quat::from_axis_angle(self.front, roll_rad);

        self.right = roll_rotation * right_base;
        self.up = roll_rotation * up_base;
    }

    /// Generates the matrix required by Vulkan shaders to render the scene from this perspective
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.front, self.up)
    }
}