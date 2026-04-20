// src/geometrical_shapes/transform.rs

use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Generates the final 4x4 Transformation Matrix that the Vertex Shader uses
    /// to move the model from "Local Space" into "World Space".
    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Helper to quickly set location
    pub fn with_translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.translation = Vec3::new(x, y, z);
        self
    }

    /// Helper to quickly set scale
    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = Vec3::new(x, y, z);
        self
    }

    /// Helper to quickly set rotation using standard Euler angles (Pitch, Yaw, Roll)
    pub fn with_rotation_euler(mut self, pitch: f32, yaw: f32, roll: f32) -> Self {
        self.rotation = Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll);
        self
    }
}