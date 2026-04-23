// src/vulkan_logic/renderers/master/shadow_pass.rs

use anyhow::Result;
use ash::vk;
use super::renderer::MasterRenderer;
use super::data_prep::FrameData;
use crate::vulkan_logic::passes::shadow_pass::SHADOW_MAP_DIM;
use crate::vulkan_logic::pipelines::main_pipeline::PushConstants;

impl MasterRenderer {
    /// Renders purely the depth logic to formulate accurate shadow cascades
    pub(crate) fn record_shadow_pass(&self, frame_data: &FrameData) -> Result<()> {
        unsafe {
            let shadow_clear_values = [vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }];
            
            // Initiate the strictly isolated depth render pass
            let shadow_render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.shadow_pass.render_pass)
                .framebuffer(self.shadow_pass.framebuffer)
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: SHADOW_MAP_DIM, height: SHADOW_MAP_DIM } })
                .clear_values(&shadow_clear_values);

            self.context.device.cmd_begin_render_pass(self.sync.command_buffer, &shadow_render_pass_info, vk::SubpassContents::INLINE);

            // Establish exact dimensional bindings
            let shadow_viewport = vk::Viewport { x: 0.0, y: 0.0, width: SHADOW_MAP_DIM as f32, height: SHADOW_MAP_DIM as f32, min_depth: 0.0, max_depth: 1.0 };
            self.context.device.cmd_set_viewport(self.sync.command_buffer, 0, std::slice::from_ref(&shadow_viewport));

            let shadow_scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: SHADOW_MAP_DIM, height: SHADOW_MAP_DIM } };
            self.context.device.cmd_set_scissor(self.sync.command_buffer, 0, std::slice::from_ref(&shadow_scissor));

            self.context.device.cmd_bind_pipeline(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.shadow_pipeline);
            
            // Connect geometry dynamically
            self.context.device.cmd_bind_vertex_buffers(self.sync.command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
            self.context.device.cmd_bind_index_buffer(self.sync.command_buffer, self.index_buffer.buffer, 0, vk::IndexType::UINT32);

            // Cycle through and render depth for all visible geometry
            for call in &frame_data.draw_calls {
                let pc = PushConstants { 
                    view_proj: frame_data.light_space_matrix, 
                    model: call.transform, 
                    light_space_matrix: frame_data.light_space_matrix 
                };
                let pc_bytes = std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<PushConstants>());
                self.context.device.cmd_push_constants(self.sync.command_buffer, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, pc_bytes);
                self.context.device.cmd_draw_indexed(self.sync.command_buffer, call.index_count, 1, call.index_start, call.vertex_offset, 0);
            }
            
            self.context.device.cmd_end_render_pass(self.sync.command_buffer);
        }
        Ok(())
    }
}