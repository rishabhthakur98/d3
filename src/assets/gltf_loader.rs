// src/assets/gltf_loader.rs

use anyhow::{anyhow, Result};
use glam::{Quat, Vec3};
use super::model::{Mesh, Model, Transform, Vertex, MaterialConfig, TextureData};

/// Safely extracts and parses `.glb` / `.gltf` data into unified Vulkan memory buffers
pub fn load_gltf_asset(
    file_path: &str,
    location: Vec3,
    orientation: Vec3, // Pitch, Yaw, Roll in Radians
    scale: Vec3,
) -> Result<Model> {
    
    // 1. Ingest the file using full feature flags and parse textures
    let (document, buffers, images) = gltf::import(file_path)
        .map_err(|e| anyhow!("Failed to load GLTF asset from {}: {}", file_path, e))?;

    let mut loaded_meshes = Vec::new();

    // 2. Iterate through local mesh hierarchies
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

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

            // ----------------------------------------------------------------------
            // FIX: Texture Baking directly into Vertex Colors (CPU-Side)
            // Because the Vulkan architecture currently does not use Image Descriptors
            // for meshes, we manually sample the texture on the CPU and bake it!
            // ----------------------------------------------------------------------
            let gltf_mat = primitive.material();
            let pbr = gltf_mat.pbr_metallic_roughness();
            let base_color_factor = pbr.base_color_factor();

            // Safely attempt to resolve the texture image associated with the base color
            let texture_image_data = pbr.base_color_texture().and_then(|info| {
                let tex_index = info.texture().source().index();
                images.get(tex_index)
            });

            // Extract any existing vertex colors the model might natively possess
            let vertex_colors: Option<Vec<[f32; 4]>> = reader.read_colors(0).map(|iter| iter.into_rgba_f32().collect());

            let mut colors = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                // Start with the raw vertex color (or pure white if none exists)
                let base_vc = match &vertex_colors {
                    Some(vc) => vc[i],
                    None => [1.0, 1.0, 1.0, 1.0],
                };

                // Sample the texture at this exact vertex UV coordinate
                let tex_color = match texture_image_data {
                    Some(img) => {
                        let raw_color = sample_texture(img, uvs[i]);
                        // GLTF textures are encoded in SRGB space. We must linearize them
                        // so the lighting shader mathematically interacts with them correctly!
                        [
                            srgb_to_linear(raw_color[0]),
                            srgb_to_linear(raw_color[1]),
                            srgb_to_linear(raw_color[2]),
                            raw_color[3], // Alpha remains untouched
                        ]
                    },
                    None => [1.0, 1.0, 1.0, 1.0],
                };

                // The GLTF spec demands: FinalColor = VertexColor * BaseColorFactor * TextureColor
                colors.push([
                    base_vc[0] * base_color_factor[0] * tex_color[0],
                    base_vc[1] * base_color_factor[1] * tex_color[1],
                    base_vc[2] * base_color_factor[2] * tex_color[2],
                    base_vc[3] * base_color_factor[3] * tex_color[3],
                ]);
            }

            // 3. Assemble mathematically aligned GPU Vertices
            let mut vertices = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                vertices.push(Vertex {
                    position: positions[i],
                    color: colors[i], // Now fully loaded with Texture & Material data!
                    normal: normals[i],
                    uv: uvs[i],
                });
            }

            let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                iter.into_u32().collect()
            } else {
                (0..vertices.len() as u32).collect()
            };

            // 4. Safely extract highly configurable PBR material bounds
            let mut material = MaterialConfig {
                base_color_factor,
                emissive_factor: gltf_mat.emissive_factor(),
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                alpha_cutoff: gltf_mat.alpha_cutoff(),
                alpha_mode: match gltf_mat.alpha_mode() {
                    gltf::material::AlphaMode::Opaque => "OPAQUE".to_string(),
                    gltf::material::AlphaMode::Mask => "MASK".to_string(),
                    gltf::material::AlphaMode::Blend => "BLEND".to_string(),
                },
                double_sided: gltf_mat.double_sided(),
                unlit: gltf_mat.unlit(),
                has_base_color_texture: pbr.base_color_texture().is_some(),
                has_normal_map: gltf_mat.normal_texture().is_some(),
                has_metallic_roughness_texture: pbr.metallic_roughness_texture().is_some(),
                has_emissive_texture: gltf_mat.emissive_texture().is_some(),
                has_occlusion_texture: gltf_mat.occlusion_texture().is_some(),
                ..Default::default()
            };

            // Safely parse Khronos Extensions (If present in the .GLB)
            if let Some(transmission) = gltf_mat.transmission() {
                material.transmission_factor = transmission.transmission_factor();
            }
            if let Some(ior) = gltf_mat.ior() {
                material.ior = ior;
            }
            if let Some(emissive_strength) = gltf_mat.emissive_strength() {
                material.emissive_strength = emissive_strength;
            }

            loaded_meshes.push(Mesh { vertices, indices, material });
        }
    }

    // 5. Extract raw image arrays (Textures, Normal maps) to hand off to Vulkan
    let mut parsed_images = Vec::with_capacity(images.len());
    for img in images {
        parsed_images.push(TextureData {
            pixels: img.pixels,
            width: img.width,
            height: img.height,
        });
    }

    // 6. Calculate final space transformations
    let rotation = Quat::from_euler(glam::EulerRot::XYZ, orientation.x, orientation.y, orientation.z);
    let transform = Transform {
        translation: location,
        rotation,
        scale,
    };

    Ok(Model {
        meshes: loaded_meshes,
        transform,
        images: parsed_images,
    })
}

// --------------------------------------------------------------------------------
// INTERNAL HELPERS FOR CPU-SIDE TEXTURE SAMPLING
// --------------------------------------------------------------------------------

/// Mathematically converts standard SRGB color values into Linear Space for accurate lighting calculations
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        f32::powf((c + 0.055) / 1.055, 2.4)
    }
}

/// Reads raw pixel arrays mapped to a specific UV coordinate dynamically
fn sample_texture(image: &gltf::image::Data, uv: [f32; 2]) -> [f32; 4] {
    // Prevent panics by forcing the UV coordinates into a safe 0.0 - 1.0 repeating grid
    let u = uv[0].rem_euclid(1.0);
    let v = uv[1].rem_euclid(1.0); // Vulkan and GLTF align on Top-Left UV origin

    let x = (u * (image.width as f32)).clamp(0.0, (image.width.saturating_sub(1)) as f32) as usize;
    let y = (v * (image.height as f32)).clamp(0.0, (image.height.saturating_sub(1)) as f32) as usize;

    let pixel_index = y * (image.width as usize) + x;

    // Extremely safe byte-extraction helper to obey strict unwrapping constraints
    let get_byte = |idx: usize| -> u8 {
        if idx < image.pixels.len() {
            image.pixels[idx]
        } else {
            255
        }
    };

    match image.format {
        gltf::image::Format::R8 => {
            let r = get_byte(pixel_index) as f32 / 255.0;
            [r, r, r, 1.0]
        },
        gltf::image::Format::R8G8 => {
            let idx = pixel_index * 2;
            let r = get_byte(idx) as f32 / 255.0;
            let g = get_byte(idx + 1) as f32 / 255.0;
            [r, g, 0.0, 1.0]
        },
        gltf::image::Format::R8G8B8 => {
            let idx = pixel_index * 3;
            let r = get_byte(idx) as f32 / 255.0;
            let g = get_byte(idx + 1) as f32 / 255.0;
            let b = get_byte(idx + 2) as f32 / 255.0;
            [r, g, b, 1.0]
        },
        gltf::image::Format::R8G8B8A8 => {
            let idx = pixel_index * 4;
            let r = get_byte(idx) as f32 / 255.0;
            let g = get_byte(idx + 1) as f32 / 255.0;
            let b = get_byte(idx + 2) as f32 / 255.0;
            let a = get_byte(idx + 3) as f32 / 255.0;
            [r, g, b, a]
        },
        gltf::image::Format::R16G16B16 => {
            let idx = pixel_index * 6;
            let r = u16::from_le_bytes([get_byte(idx), get_byte(idx+1)]) as f32 / 65535.0;
            let g = u16::from_le_bytes([get_byte(idx+2), get_byte(idx+3)]) as f32 / 65535.0;
            let b = u16::from_le_bytes([get_byte(idx+4), get_byte(idx+5)]) as f32 / 65535.0;
            [r, g, b, 1.0]
        },
        gltf::image::Format::R16G16B16A16 => {
            let idx = pixel_index * 8;
            let r = u16::from_le_bytes([get_byte(idx), get_byte(idx+1)]) as f32 / 65535.0;
            let g = u16::from_le_bytes([get_byte(idx+2), get_byte(idx+3)]) as f32 / 65535.0;
            let b = u16::from_le_bytes([get_byte(idx+4), get_byte(idx+5)]) as f32 / 65535.0;
            let a = u16::from_le_bytes([get_byte(idx+6), get_byte(idx+7)]) as f32 / 65535.0;
            [r, g, b, a]
        },
        _ => [1.0, 1.0, 1.0, 1.0] // Unknown or unsupported formats gracefully default to white
    }
}