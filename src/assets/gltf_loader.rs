// src/assets/gltf_loader.rs

use anyhow::{anyhow, Result};
use glam::{Quat, Vec3};
use super::model::{Mesh, Model, Transform, Vertex};

/// Safely extracts and parses `.glb` / `.gltf` data into unified Vulkan memory buffers
pub fn load_gltf_asset(
    file_path: &str,
    location: Vec3,
    orientation: Vec3, // Pitch, Yaw, Roll in Radians
    scale: Vec3,
) -> Result<Model> {
    
    // 1. Ingest the file using full feature flags
    let (document, buffers, _images) = gltf::import(file_path)
        .map_err(|e| anyhow!("Failed to load GLTF asset from {}: {}", file_path, e))?;

    let mut loaded_meshes = Vec::new();

    // 2. Iterate through local mesh hierarchies
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Ensure we handle missing components safely without panics
            let positions: Vec<[f32; 3]> = if let Some(iter) = reader.read_positions() {
                iter.collect()
            } else {
                continue; 
            };

            let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
                iter.collect()
            } else {
                vec![[0.0, 1.0, 0.0]; positions.len()]
            };

            let uvs: Vec<[f32; 2]> = if let Some(iter) = reader.read_tex_coords(0) {
                iter.into_f32().collect()
            } else {
                vec![[0.0, 0.0]; positions.len()]
            };

            let colors: Vec<[f32; 4]> = if let Some(iter) = reader.read_colors(0) {
                iter.into_rgba_f32().collect()
            } else {
                vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
            };

            // 3. Assemble mathematically aligned GPU Vertices
            let mut vertices = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                vertices.push(Vertex {
                    position: positions[i],
                    color: colors[i],
                    normal: normals[i],
                    uv: uvs[i],
                });
            }

            let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                iter.into_u32().collect()
            } else {
                (0..vertices.len() as u32).collect()
            };

            loaded_meshes.push(Mesh { vertices, indices });
        }
    }

    // 4. Calculate final space transformations
    let rotation = Quat::from_euler(glam::EulerRot::XYZ, orientation.x, orientation.y, orientation.z);
    let transform = Transform {
        translation: location,
        rotation,
        scale,
    };

    Ok(Model {
        meshes: loaded_meshes,
        transform,
    })
}