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

/// Highly configurable material settings extracted directly from the GLTF/GLB asset
#[derive(Clone, Debug)]
pub struct MaterialConfig {
    pub base_color_factor: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: String,
    pub double_sided: bool,

    // Advanced KHR Extracted Extensions
    pub unlit: bool,
    pub ior: f32,
    pub transmission_factor: f32,
    pub emissive_strength: f32,
    
    // Texture Identifiers
    pub has_base_color_texture: bool,
    pub has_normal_map: bool,
    pub has_metallic_roughness_texture: bool,
    pub has_emissive_texture: bool,
    pub has_occlusion_texture: bool,
}

impl Default for MaterialConfig {
    fn default() -> Self {
        Self {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            emissive_factor: [0.0, 0.0, 0.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            alpha_cutoff: None,
            alpha_mode: "OPAQUE".to_string(),
            double_sided: false,
            unlit: false,
            ior: 1.5,
            transmission_factor: 0.0,
            emissive_strength: 1.0,
            has_base_color_texture: false,
            has_normal_map: false,
            has_metallic_roughness_texture: false,
            has_emissive_texture: false,
            has_occlusion_texture: false,
        }
    }
}

/// Raw Image data extracted from the GLB for the GPU descriptor sets
#[derive(Clone, Debug)]
pub struct TextureData {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Extracted GPU buffers
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: MaterialConfig, // Newly extracted material!
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
    pub images: Vec<TextureData>, // Holds all the parsed normal maps and albedos
}