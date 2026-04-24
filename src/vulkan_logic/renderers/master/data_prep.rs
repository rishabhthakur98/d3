// src/vulkan_logic/renderers/master/data_prep.rs

use anyhow::{anyhow, Result};
use super::renderer::{MasterRenderer, DrawCall};
use crate::assets::model::Model;

use crate::lights::global::directional::GlobalLight;
use crate::lights::spawnable::spot::SpotLight;
use crate::lights::spawnable::point::PointLight;
// PHASE 2 FIX: Migrate data prep to push to the new SSBO structs
use crate::lights::core::ssbo::{LightSSBO, SpotLightData, GlobalLightData, PointLightData, FogVolumeData};
use crate::volumetrics::fog_config::FogVolume;
use crate::lights::core::config::{MAX_GLOBAL_LIGHTS, MAX_SPOT_LIGHTS, MAX_POINT_LIGHTS, MAX_FOG_VOLUMES};

pub(crate) struct FrameData {
    pub draw_calls: Vec<DrawCall>,
    pub light_space_matrix: glam::Mat4,
    pub primary_sun_dir: glam::Vec3,
    pub primary_sun_color: [f32; 3],
    pub primary_sun_intensity: f32,
}

impl MasterRenderer {
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
        visible_objects: &[Model], 
        fog_volumes: &[FogVolume],
    ) -> Result<FrameData> {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut draw_calls = Vec::new();
        let mut light_space_matrix = glam::Mat4::IDENTITY;

        let mut primary_sun_dir = glam::Vec3::new(0.0, -1.0, 0.0);
        let mut primary_sun_color = [1.0, 1.0, 1.0];
        let mut primary_sun_intensity = 0.0;

        if is_playing {
            // Initialize SSBO rather than UBO
            let mut ssbo = LightSSBO {
                ambient_color: glam::Vec4::new(ambient_color[0], ambient_color[1], ambient_color[2], ambient_intensity),
                camera_pos: glam::Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0),
                global_count: 0,
                spot_count: 0,
                point_count: 0,
                fog_count: 0,
                
                global_lights: [GlobalLightData::default(); MAX_GLOBAL_LIGHTS],
                spot_lights: [SpotLightData::default(); MAX_SPOT_LIGHTS],
                point_lights: [PointLightData::default(); MAX_POINT_LIGHTS],
                fog_volumes: [FogVolumeData::default(); MAX_FOG_VOLUMES],
            };

            let mut shadow_caster_dir = None;

            for gl in global_lights {
                if ssbo.global_count < MAX_GLOBAL_LIGHTS as u32 {
                    let idx = ssbo.global_count as usize;
                    ssbo.global_lights[idx] = GlobalLightData {
                        direction: glam::Vec4::new(gl.direction.x, gl.direction.y, gl.direction.z, gl.intensity),
                        color: glam::Vec4::new(gl.color[0], gl.color[1], gl.color[2], if gl.cast_shadows { 1.0 } else { 0.0 }),
                    };
                    ssbo.global_count += 1;
                    
                    if gl.cast_shadows && shadow_caster_dir.is_none() {
                        shadow_caster_dir = Some(gl.direction.normalize());
                        primary_sun_dir = gl.direction.normalize();
                        primary_sun_color = gl.color;
                        primary_sun_intensity = gl.intensity;
                    }
                }
            }

            if let Some(sun_dir) = shadow_caster_dir {
                let target = glam::Vec3::new(camera_pos.x, 0.0, camera_pos.z);
                let light_pos = target - (sun_dir * 100.0); 
                let up = glam::Vec3::new(0.0, 1.0, 0.0);
                let view_matrix = glam::Mat4::look_at_rh(light_pos, target, up);
                let ortho_size = 50.0;
                let mut proj_matrix = glam::Mat4::orthographic_rh(-ortho_size, ortho_size, -ortho_size, ortho_size, 0.1, 300.0);
                proj_matrix.y_axis.y *= -1.0; 
                light_space_matrix = proj_matrix * view_matrix;
            }

            for model in visible_objects {
                let transform_matrix = model.transform.get_model_matrix();
                for mesh in &model.meshes {
                    let vertex_offset = all_vertices.len() as i32;
                    let index_start = all_indices.len() as u32;
                    let index_count = mesh.indices.len() as u32;

                    all_vertices.extend_from_slice(&mesh.vertices);
                    all_indices.extend_from_slice(&mesh.indices);
                    draw_calls.push(DrawCall { index_start, index_count, vertex_offset, transform: transform_matrix });
                }
            }

            for spot in spot_lights {
                if ssbo.spot_count < MAX_SPOT_LIGHTS as u32 {
                    let idx = ssbo.spot_count as usize;
                    ssbo.spot_lights[idx] = SpotLightData {
                        position: glam::Vec4::new(spot.position.x, spot.position.y, spot.position.z, spot.range),
                        direction: glam::Vec4::new(spot.direction.x, spot.direction.y, spot.direction.z, spot.intensity),
                        color: glam::Vec4::new(spot.color[0], spot.color[1], spot.color[2], spot.inner_cone_angle),
                        params: glam::Vec4::new(spot.outer_cone_angle, if spot.cast_shadows { 1.0 } else { 0.0 }, 0.0, 0.0),
                    };
                    ssbo.spot_count += 1;
                }
            }

            for point in point_lights {
                if ssbo.point_count < MAX_POINT_LIGHTS as u32 {
                    let idx = ssbo.point_count as usize;
                    ssbo.point_lights[idx] = PointLightData {
                        position: glam::Vec4::new(point.position.x, point.position.y, point.position.z, point.range),
                        color: glam::Vec4::new(point.color[0], point.color[1], point.color[2], point.intensity),
                    };
                    ssbo.point_count += 1;
                }
            }

            for fog in fog_volumes {
                if ssbo.fog_count < MAX_FOG_VOLUMES as u32 {
                    let idx = ssbo.fog_count as usize;
                    ssbo.fog_volumes[idx] = FogVolumeData {
                        min_bounds: glam::Vec4::new(fog.min_bounds.x, fog.min_bounds.y, fog.min_bounds.z, 0.0),
                        max_bounds: glam::Vec4::new(fog.max_bounds.x, fog.max_bounds.y, fog.max_bounds.z, 0.0),
                        color_density: glam::Vec4::new(fog.color[0], fog.color[1], fog.color[2], fog.density),
                    };
                    ssbo.fog_count += 1;
                }
            }

            let allocator = match self.context.allocator.as_ref() {
                Some(alloc) => alloc,
                None => return Err(anyhow!("Memory allocator missing during data prep")),
            };

            self.vertex_buffer.upload_data(allocator, &all_vertices)?;
            self.index_buffer.upload_data(allocator, &all_indices)?;
            self.ssbo_buffer.upload_data(allocator, &[ssbo])?; // Push to SSBO buffer safely
        }

        Ok(FrameData { draw_calls, light_space_matrix, primary_sun_dir, primary_sun_color, primary_sun_intensity })
    }
}