// src/vulkan_logic/renderers/master/main_pass.rs

use anyhow::Result;
use ash::vk;

use super::renderer::MasterRenderer;
use super::data_prep::FrameData;

use crate::vulkan_logic::pipelines::main_pipeline::PushConstants;
use crate::skybox::config::SkyboxConfig;
use crate::clouds::config::CloudConfig;
use crate::water::config::RiverConfig;
use crate::smoke::emitter::SmokeEmitter;
use crate::fire::emitter::FireEmitter;
use crate::weather::emitter::WeatherEmitter;

impl MasterRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_main_pass(
        &mut self,
        img_idx: u32,
        is_playing: bool,
        camera_pos: glam::Vec3,
        camera_view_matrix: glam::Mat4,
        frame_data: &FrameData,
        skybox_config: &SkyboxConfig,
        cloud_config: &CloudConfig,
        river_configs: &[RiverConfig],
        smoke_emitters: &[SmokeEmitter],
        fire_emitters: &[FireEmitter],
        weather_emitter: &WeatherEmitter,
        clipped_primitives: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
        time: f32,
    ) -> Result<()> {
        unsafe {
            let clear_color = [0.0, 0.0, 0.0, 1.0];
            let clear_values = [
                vk::ClearValue { color: vk::ClearColorValue { float32: clear_color } },
                vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }, 
            ]; 
            
            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.swapchain_mgr.render_pass)
                .framebuffer(self.swapchain_mgr.framebuffers[img_idx as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_mgr.extent })
                .clear_values(&clear_values);

            self.context.device.cmd_begin_render_pass(self.sync.command_buffer, &render_pass_info, vk::SubpassContents::INLINE);

            if is_playing {
                let viewport = vk::Viewport { x: 0.0, y: 0.0, width: self.swapchain_mgr.extent.width as f32, height: self.swapchain_mgr.extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
                self.context.device.cmd_set_viewport(self.sync.command_buffer, 0, std::slice::from_ref(&viewport));

                let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_mgr.extent };
                self.context.device.cmd_set_scissor(self.sync.command_buffer, 0, std::slice::from_ref(&scissor));

                self.skybox_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    &self.swapchain_mgr.extent,
                    skybox_config,
                    camera_view_matrix,
                )?;

                let aspect = self.swapchain_mgr.extent.width as f32 / self.swapchain_mgr.extent.height as f32;
                let mut proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);
                proj.y_axis.y *= -1.0; 
                let view_proj = proj * camera_view_matrix;
                let inv_view_proj = view_proj.inverse();

                self.cloud_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    cloud_config,
                    inv_view_proj,
                    camera_pos,
                    frame_data.primary_sun_dir,
                    frame_data.primary_sun_color,
                    frame_data.primary_sun_intensity,
                    time,
                )?;

                if !frame_data.draw_calls.is_empty() {
                    self.context.device.cmd_bind_pipeline(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline);
                    self.context.device.cmd_bind_vertex_buffers(self.sync.command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
                    self.context.device.cmd_bind_index_buffer(self.sync.command_buffer, self.index_buffer.buffer, 0, vk::IndexType::UINT32);
                    self.context.device.cmd_bind_descriptor_sets(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.layout, 0, std::slice::from_ref(&self.descriptor_set), &[]);

                    for call in &frame_data.draw_calls {
                        let pc = PushConstants { view_proj, model: call.transform, light_space_matrix: frame_data.light_space_matrix };
                        let pc_bytes = std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<PushConstants>());
                        self.context.device.cmd_push_constants(self.sync.command_buffer, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, pc_bytes);
                        self.context.device.cmd_draw_indexed(self.sync.command_buffer, call.index_count, 1, call.index_start, call.vertex_offset, 0);
                    }
                }

                // Array driven passes
                self.river_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    river_configs,
                    view_proj,
                    camera_pos,
                    frame_data.primary_sun_dir,
                    frame_data.primary_sun_color,
                    frame_data.primary_sun_intensity,
                    time,
                )?;

                self.fire_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    fire_emitters,
                    view_proj,
                    camera_view_matrix,
                )?;

                self.smoke_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    smoke_emitters,
                    view_proj,
                    camera_view_matrix,
                )?;

                self.weather_system.draw(
                    &self.context,
                    self.sync.command_buffer,
                    weather_emitter,
                    view_proj,
                    camera_view_matrix,
                    camera_pos,
                    time,
                )?;
            }

            if !clipped_primitives.is_empty() {
                // Return mapped error string instead of unwrapping 
                self.egui_renderer.cmd_draw(self.sync.command_buffer, self.swapchain_mgr.extent, pixels_per_point, clipped_primitives)
                    .map_err(|e| anyhow::anyhow!("Egui draw error: {}", e))?;
            }
            
            self.context.device.cmd_end_render_pass(self.sync.command_buffer);
        }
        Ok(())
    }
}