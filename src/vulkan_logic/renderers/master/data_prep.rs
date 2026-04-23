// src/vulkan_logic/renderers/master/data_prep.rs

use anyhow::{anyhow, Result};
use super::renderer::{MasterRenderer, DrawCall};
use crate::geometrical_shapes::game_object::GameObject;
use crate::light::global::GlobalLight;
use crate::light::spot::SpotLight;
use crate::light::point::PointLight;
use crate::light::ubo::{LightUBO, SpotLightData, GlobalLightData, PointLightData};
use crate::volumetrics::fog_config::FogConfig;

/// A segregated struct strictly to hold calculated frame metrics
pub(crate) struct FrameData {
    pub draw_calls: Vec<DrawCall>,
    pub light_space_matrix: glam::Mat4,
    pub primary_sun_dir: glam::Vec3,
    pub primary_sun_color: [f32; 3],
    pub primary_sun_intensity: f32,
}

impl MasterRenderer {
    /// Consolidates all math, geometry extraction, and GPU buffer uploads.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_data(
        &mut self,
        is_playing: bool,
        camera_pos: glam::Vec3,
        ambient_color: [f32; 3],
        ambient_intensity: f32,
        global_lights: &[GlobalLight],
        spot_lights: &[SpotLight],
        point_lights: &[PointLight],
        visible_objects: &[GameObject],
        fog_config: &FogConfig,
    ) -> Result<FrameData> {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut draw_calls = Vec::new();
        let mut light_space_matrix = glam::Mat4::IDENTITY;

        let mut primary_sun_dir = glam::Vec3::new(0.0, -1.0, 0.0);
        let mut primary_sun_color = [1.0, 1.0, 1.0];
        let mut primary_sun_intensity = 0.0;

        if is_playing {
            // Rebuild the master lighting struct for this specific frame
            let mut ubo = LightUBO {
                ambient_color: glam::Vec4::new(ambient_color[0], ambient_color[1], ambient_color[2], ambient_intensity),
                camera_pos: glam::Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0),
                global_count: 0,
                spot_count: 0,
                point_count: 0,
                _pad: 0,
                
                fog_color: glam::Vec4::new(fog_config.color[0], fog_config.color[1], fog_config.color[2], fog_config.global_density),
                fog_params: glam::Vec4::new(fog_config.height_falloff, fog_config.height_offset, fog_config.volumetric_scattering, 0.0),
                
                global_lights: [GlobalLightData::default(); 4],
                spot_lights: [SpotLightData::default(); 10],
                point_lights: [PointLightData::default(); 10],
            };

            let mut shadow_caster_dir = None;

            // Load suns and moons
            for gl in global_lights {
                if ubo.global_count < 4 {
                    let idx = ubo.global_count as usize;
                    ubo.global_lights[idx] = GlobalLightData {
                        direction: glam::Vec4::new(gl.direction.x, gl.direction.y, gl.direction.z, gl.intensity),
                        color: glam::Vec4::new(gl.color[0], gl.color[1], gl.color[2], if gl.cast_shadows { 1.0 } else { 0.0 }),
                    };
                    ubo.global_count += 1;
                    
                    if gl.cast_shadows && shadow_caster_dir.is_none() {
                        shadow_caster_dir = Some(gl.direction.normalize());
                        primary_sun_dir = gl.direction.normalize();
                        primary_sun_color = gl.color;
                        primary_sun_intensity = gl.intensity;
                    }
                }
            }

            // Calculate the directional lighting shadow projection logic
            if let Some(sun_dir) = shadow_caster_dir {
                let target = glam::Vec3::new(camera_pos.x, 0.0, camera_pos.z);
                let light_pos = target - (sun_dir * 100.0); 
                let up = glam::Vec3::new(0.0, 1.0, 0.0);
                let view_matrix = glam::Mat4::look_at_rh(light_pos, target, up);
                let ortho_size = 50.0;
                let mut proj_matrix = glam::Mat4::orthographic_rh(-ortho_size, ortho_size, -ortho_size, ortho_size, 0.1, 300.0);
                proj_matrix.y_axis.y *= -1.0; // Vulkan Y-flip correction
                light_space_matrix = proj_matrix * view_matrix;
            }

            // Unify geometry into unified buffers to dramatically reduce draw overhead
            for obj in visible_objects {
                let vertex_offset = all_vertices.len() as i32;
                let index_start = all_indices.len() as u32;
                let index_count = obj.mesh.indices.len() as u32;

                all_vertices.extend_from_slice(&obj.mesh.vertices);
                all_indices.extend_from_slice(&obj.mesh.indices);
                draw_calls.push(DrawCall { index_start, index_count, vertex_offset, transform: obj.transform.get_model_matrix() });
            }

            // Extract localized spotlights (flashlights, lamps)
            for spot in spot_lights {
                if ubo.spot_count < 10 {
                    let idx = ubo.spot_count as usize;
                    ubo.spot_lights[idx] = SpotLightData {
                        position: glam::Vec4::new(spot.position.x, spot.position.y, spot.position.z, spot.range),
                        direction: glam::Vec4::new(spot.direction.x, spot.direction.y, spot.direction.z, spot.intensity),
                        color: glam::Vec4::new(spot.color[0], spot.color[1], spot.color[2], spot.inner_cone_angle),
                        params: glam::Vec4::new(spot.outer_cone_angle, if spot.cast_shadows { 1.0 } else { 0.0 }, 0.0, 0.0),
                    };
                    ubo.spot_count += 1;
                }
            }

            // Extract localized spherical points (torches, fireflies)
            for point in point_lights {
                if ubo.point_count < 10 {
                    let idx = ubo.point_count as usize;
                    ubo.point_lights[idx] = PointLightData {
                        position: glam::Vec4::new(point.position.x, point.position.y, point.position.z, point.range),
                        color: glam::Vec4::new(point.color[0], point.color[1], point.color[2], point.intensity),
                    };
                    ubo.point_count += 1;
                }
            }

            // Synchronously beam data to the GPU immediately
            let allocator = match self.context.allocator.as_ref() {
                Some(alloc) => alloc,
                None => return Err(anyhow!("Memory allocator missing during data prep")),
            };

            self.vertex_buffer.upload_data(allocator, &all_vertices)?;
            self.index_buffer.upload_data(allocator, &all_indices)?;
            self.uniform_buffer.upload_data(allocator, &[ubo])?;
        }

        Ok(FrameData {
            draw_calls,
            light_space_matrix,
            primary_sun_dir,
            primary_sun_color,
            primary_sun_intensity,
        })
    }
}