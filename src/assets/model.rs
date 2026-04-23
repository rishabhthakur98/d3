// src/assets/model.rs

use glam::{Mat4, Quat, Vec3};

/// A memory-aligned vertex strictly mirroring the Vulkan GPU layout expectations
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3], 
    pub color: [f32; 4],    
    pub normal: [f32; 3],   
    pub uv: [f32; 2],       
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            color: [1.0; 4],
            normal: [0.0, 1.0, 0.0],
            uv: [0.0; 2],
        }
    }
}

/// Extracted GPU buffers
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn get_model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// A comprehensive game model parsed dynamically from GLTF/GLB files
#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub transform: Transform,
}