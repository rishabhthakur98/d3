// src/vulkan_logic/renderer.rs
use anyhow::{anyhow, Result};
use ash::vk;
use egui_ash_renderer::Renderer as EguiRenderer;
use egui_ash_renderer::Options as EguiOptions;
use winit::window::Window;
use std::sync::Arc;

use super::context::VulkanContext;
use super::swapchain_manager::SwapchainManager;
use super::sync_objects::SyncObjects;
use super::pipeline::{VulkanPipeline, PushConstants};
use super::gpu_buffers::DynamicBuffer;
use super::shadow_pass::{ShadowPass, SHADOW_MAP_DIM};

use crate::geometrical_shapes::game_object::GameObject;

use crate::light::global::GlobalLight;
use crate::light::spot::SpotLight; // NEW: Direct import of SpotLight
use crate::light::ubo::{LightUBO, SpotLightData, GlobalLightData};

struct DrawCall {
    index_start: u32,
    index_count: u32,
    vertex_offset: i32,
    transform: glam::Mat4,
}

pub struct VulkanRenderer {
    pub is_resized: bool,
    pub egui_renderer: EguiRenderer,
    context: Arc<VulkanContext>,
    swapchain_mgr: SwapchainManager,
    sync: SyncObjects,
    shadow_pass: ShadowPass, 
    pipeline: VulkanPipeline,
    vertex_buffer: DynamicBuffer,
    index_buffer: DynamicBuffer,
    uniform_buffer: DynamicBuffer,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
}

impl VulkanRenderer {
    pub fn new(context: Arc<VulkanContext>, window: &Window) -> Result<Self> {
        let swapchain_mgr = SwapchainManager::new(&context, window)?;
        let sync = SyncObjects::new(&context)?;

        let egui_renderer = EguiRenderer::with_default_allocator(
            &context.instance, context.physical_device, context.device.clone(), swapchain_mgr.render_pass,
            EguiOptions { srgb_framebuffer: false, ..Default::default() },
        ).map_err(|e| anyhow!("Failed to initialize egui: {}", e))?;

        let shadow_pass = ShadowPass::new(&context)?;
        let pipeline = VulkanPipeline::new(&context, swapchain_mgr.render_pass, shadow_pass.render_pass)?;
        
        let allocator = match context.allocator.as_ref() {
            Some(alloc) => alloc,
            None => return Err(anyhow!("Vulkan memory allocator was not initialized")),
        };

        let vertex_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<crate::geometrical_shapes::triangle::Vertex>() * 10000, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let index_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<u32>() * 10000, vk::BufferUsageFlags::INDEX_BUFFER)?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1),
        ];
            
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }.map_err(|e| anyhow!("Failed to create descriptor pool: {}", e))?;

        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(std::slice::from_ref(&pipeline.descriptor_set_layout));
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }.map_err(|e| anyhow!("Failed to allocate descriptor sets: {}", e))?[0];

        let uniform_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<LightUBO>(), vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let uniform_buffer_info = vk::DescriptorBufferInfo::default().buffer(uniform_buffer.buffer).offset(0).range(std::mem::size_of::<LightUBO>() as u64);
        let shadow_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL).image_view(shadow_pass.depth_view).sampler(shadow_pass.sampler);

        let write_sets = [
            vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(std::slice::from_ref(&uniform_buffer_info)),
            vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(1).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&shadow_image_info)),
        ];
            
        unsafe { context.device.update_descriptor_sets(&write_sets, &[]) };

        Ok(Self { 
            is_resized: false, egui_renderer, context, swapchain_mgr, sync, shadow_pass, pipeline, vertex_buffer, index_buffer, uniform_buffer, descriptor_pool, descriptor_set
        })
    }

    pub fn draw_frame(
        &mut self, 
        window: &Window, 
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        pixels_per_point: f32,
        is_playing: bool,
        camera_pos: glam::Vec3,
        camera_view_matrix: glam::Mat4,           
        ambient_color: [f32; 3],
        ambient_intensity: f32,
        global_lights: &[GlobalLight],
        spot_lights: &[SpotLight],         // NEW: Render receives completely detached spotlight array
        visible_objects: &[GameObject],    
    ) -> Result<()> {
        
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut draw_calls = Vec::new();
        let mut light_space_matrix = glam::Mat4::IDENTITY;

        if is_playing {
            let mut ubo = LightUBO {
                ambient_color: glam::Vec4::new(ambient_color[0], ambient_color[1], ambient_color[2], ambient_intensity),
                camera_pos: glam::Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0),
                global_count: 0,
                spot_count: 0,
                _pad: [0; 2],
                global_lights: [GlobalLightData::default(); 4],
                spot_lights: [SpotLightData::default(); 10],
            };

            let mut shadow_caster_dir = None;

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

            // Iterate GameObjects ONLY to build geometry draw calls
            for obj in visible_objects {
                let vertex_offset = all_vertices.len() as i32;
                let index_start = all_indices.len() as u32;
                let index_count = obj.mesh.indices.len() as u32;

                all_vertices.extend_from_slice(&obj.mesh.vertices);
                all_indices.extend_from_slice(&obj.mesh.indices);

                draw_calls.push(DrawCall {
                    index_start, index_count, vertex_offset, transform: obj.transform.get_model_matrix(),
                });
            }

            // Iterate SpotLights independently!
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

            let allocator = match self.context.allocator.as_ref() {
                Some(alloc) => alloc,
                None => return Err(anyhow!("Memory allocator missing during draw frame")),
            };

            self.vertex_buffer.upload_data(allocator, &all_vertices)?;
            self.index_buffer.upload_data(allocator, &all_indices)?;
            self.uniform_buffer.upload_data(allocator, &[ubo])?;
        }
        
        unsafe {
            self.context.device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX).map_err(|e| anyhow!("Wait for fences failed: {}", e))?;

            if !textures_delta.set.is_empty() {
                self.egui_renderer.set_textures(self.context.graphics_queue, self.sync.command_pool, textures_delta.set.as_slice()).map_err(|e| anyhow!("Failed to upload egui textures: {}", e))?;
            }
            if !textures_delta.free.is_empty() {
                self.egui_renderer.free_textures(textures_delta.free.as_slice()).map_err(|e| anyhow!("Failed to free egui textures: {}", e))?;
            }

            let acquire_result = self.swapchain_mgr.loader.acquire_next_image(self.swapchain_mgr.swapchain, u64::MAX, self.sync.image_available, vk::Fence::null());
            let img_idx = match acquire_result {
                Ok((idx, is_suboptimal)) => {
                    if is_suboptimal || self.is_resized { self.swapchain_mgr.recreate(&self.context, window)?; self.is_resized = false; return Ok(()); }
                    idx
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { self.swapchain_mgr.recreate(&self.context, window)?; self.is_resized = false; return Ok(()); }
                Err(e) => return Err(anyhow!("Failed to acquire image: {}", e)),
            };

            self.context.device.reset_fences(&[self.sync.in_flight]).map_err(|e| anyhow!("Failed to reset fences: {}", e))?;
            self.context.device.reset_command_buffer(self.sync.command_buffer, vk::CommandBufferResetFlags::empty()).map_err(|e| anyhow!("Failed to reset command buffer: {}", e))?;
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.context.device.begin_command_buffer(self.sync.command_buffer, &begin_info).map_err(|e| anyhow!("Failed to begin command buffer: {}", e))?;
            
            if is_playing && !draw_calls.is_empty() {
                let shadow_clear_values = [vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }];
                let shadow_render_pass_info = vk::RenderPassBeginInfo::default()
                    .render_pass(self.shadow_pass.render_pass).framebuffer(self.shadow_pass.framebuffer)
                    .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: SHADOW_MAP_DIM, height: SHADOW_MAP_DIM } })
                    .clear_values(&shadow_clear_values);

                self.context.device.cmd_begin_render_pass(self.sync.command_buffer, &shadow_render_pass_info, vk::SubpassContents::INLINE);

                let shadow_viewport = vk::Viewport { x: 0.0, y: 0.0, width: SHADOW_MAP_DIM as f32, height: SHADOW_MAP_DIM as f32, min_depth: 0.0, max_depth: 1.0 };
                self.context.device.cmd_set_viewport(self.sync.command_buffer, 0, std::slice::from_ref(&shadow_viewport));

                let shadow_scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: SHADOW_MAP_DIM, height: SHADOW_MAP_DIM } };
                self.context.device.cmd_set_scissor(self.sync.command_buffer, 0, std::slice::from_ref(&shadow_scissor));

                self.context.device.cmd_bind_pipeline(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.shadow_pipeline);
                self.context.device.cmd_bind_vertex_buffers(self.sync.command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
                self.context.device.cmd_bind_index_buffer(self.sync.command_buffer, self.index_buffer.buffer, 0, vk::IndexType::UINT32);

                for call in &draw_calls {
                    let pc = PushConstants { view_proj: light_space_matrix, model: call.transform, light_space_matrix };
                    let pc_bytes = std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<PushConstants>());
                    self.context.device.cmd_push_constants(self.sync.command_buffer, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, pc_bytes);
                    self.context.device.cmd_draw_indexed(self.sync.command_buffer, call.index_count, 1, call.index_start, call.vertex_offset, 0);
                }
                self.context.device.cmd_end_render_pass(self.sync.command_buffer);
            }

            let clear_color = [0.0, 0.0, 0.0, 1.0];
            let clear_values = [
                vk::ClearValue { color: vk::ClearColorValue { float32: clear_color } },
                vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }, 
            ]; 
            
            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.swapchain_mgr.render_pass).framebuffer(self.swapchain_mgr.framebuffers[img_idx as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_mgr.extent }).clear_values(&clear_values);

            self.context.device.cmd_begin_render_pass(self.sync.command_buffer, &render_pass_info, vk::SubpassContents::INLINE);

            if is_playing && !draw_calls.is_empty() {
                let viewport = vk::Viewport { x: 0.0, y: 0.0, width: self.swapchain_mgr.extent.width as f32, height: self.swapchain_mgr.extent.height as f32, min_depth: 0.0, max_depth: 1.0 };
                self.context.device.cmd_set_viewport(self.sync.command_buffer, 0, std::slice::from_ref(&viewport));

                let scissor = vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain_mgr.extent };
                self.context.device.cmd_set_scissor(self.sync.command_buffer, 0, std::slice::from_ref(&scissor));

                self.context.device.cmd_bind_pipeline(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline);
                self.context.device.cmd_bind_vertex_buffers(self.sync.command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
                self.context.device.cmd_bind_index_buffer(self.sync.command_buffer, self.index_buffer.buffer, 0, vk::IndexType::UINT32);

                self.context.device.cmd_bind_descriptor_sets(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.layout, 0, std::slice::from_ref(&self.descriptor_set), &[]);

                let aspect = self.swapchain_mgr.extent.width as f32 / self.swapchain_mgr.extent.height as f32;
                let mut proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);
                proj.y_axis.y *= -1.0; 
                let view_proj = proj * camera_view_matrix;

                for call in draw_calls {
                    let pc = PushConstants { view_proj, model: call.transform, light_space_matrix };
                    let pc_bytes = std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<PushConstants>());
                    self.context.device.cmd_push_constants(self.sync.command_buffer, self.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, pc_bytes);
                    self.context.device.cmd_draw_indexed(self.sync.command_buffer, call.index_count, 1, call.index_start, call.vertex_offset, 0);
                }
            }

            if !clipped_primitives.is_empty() {
                self.egui_renderer.cmd_draw(self.sync.command_buffer, self.swapchain_mgr.extent, pixels_per_point, clipped_primitives).map_err(|e| anyhow!("Failed to draw egui primitives: {}", e))?;
            }
            
            self.context.device.cmd_end_render_pass(self.sync.command_buffer);
            self.context.device.end_command_buffer(self.sync.command_buffer).map_err(|e| anyhow!("Failed to end command buffer: {}", e))?;

            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default().wait_semaphores(&wait_semaphores).wait_dst_stage_mask(&wait_stages).command_buffers(&command_buffers).signal_semaphores(&signal_semaphores);
            self.context.device.queue_submit(self.context.graphics_queue, std::slice::from_ref(&submit_info), self.sync.in_flight).map_err(|e| anyhow!("Failed to submit queue: {}", e))?;
            
            let active_swapchains = [self.swapchain_mgr.swapchain];
            let active_indices = [img_idx];
            let present_info = vk::PresentInfoKHR::default().wait_semaphores(&signal_semaphores).swapchains(&active_swapchains).image_indices(&active_indices);

            let present_result = self.swapchain_mgr.loader.queue_present(self.context.present_queue, &present_info);
            match present_result {
                Ok(is_suboptimal) => { if is_suboptimal || self.is_resized { self.swapchain_mgr.recreate(&self.context, window)?; self.is_resized = false; } }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { self.swapchain_mgr.recreate(&self.context, window)?; self.is_resized = false; }
                Err(e) => return Err(anyhow!("Failed to present image: {}", e)),
            }
        }
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.context.device.device_wait_idle();
            if let Some(allocator) = self.context.allocator.as_ref() {
                self.vertex_buffer.destroy(allocator);
                self.index_buffer.destroy(allocator);
                self.uniform_buffer.destroy(allocator); 
            }
            self.context.device.destroy_descriptor_pool(self.descriptor_pool, None); 
            self.shadow_pass.destroy(&self.context); 
            self.pipeline.destroy(&self.context.device);
            self.sync.destroy(&self.context.device);
            self.swapchain_mgr.destroy(&self.context);
        }
    }
}