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

use crate::game::world01::camera::FreeformCamera;
use crate::geometrical_shapes::game_object::GameObject;

// Import our new lighting structures and config
use crate::light::ubo::{LightUBO, SpotLightData};
use crate::game::world01::ambient_light_config::{AMBIENT_LIGHT_COLOR, AMBIENT_LIGHT_INTENSITY};

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
    
    pipeline: VulkanPipeline,
    vertex_buffer: DynamicBuffer,
    index_buffer: DynamicBuffer,
    
    // NEW: Uniform Buffer resources for Lighting
    uniform_buffer: DynamicBuffer,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
}

impl VulkanRenderer {
    pub fn new(context: Arc<VulkanContext>, window: &Window) -> Result<Self> {
        let swapchain_mgr = SwapchainManager::new(&context, window)?;
        let sync = SyncObjects::new(&context)?;

        let egui_renderer = EguiRenderer::with_default_allocator(
            &context.instance,
            context.physical_device,
            context.device.clone(),
            swapchain_mgr.render_pass,
            EguiOptions {
                srgb_framebuffer: false,
                ..Default::default()
            },
        ).map_err(|e| anyhow!("Failed to initialize egui: {}", e))?;

        let pipeline = VulkanPipeline::new(&context, swapchain_mgr.render_pass)?;
        
        let allocator = match context.allocator.as_ref() {
            Some(alloc) => alloc,
            None => return Err(anyhow!("Vulkan memory allocator was not initialized")),
        };

        let vertex_buffer = DynamicBuffer::new(
            allocator,
            std::mem::size_of::<crate::geometrical_shapes::triangle::Vertex>() * 10000,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;
        
        let index_buffer = DynamicBuffer::new(
            allocator,
            std::mem::size_of::<u32>() * 10000,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;

        // --- NEW: Descriptor Pool & Uniform Buffer Allocation ---
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1);
            
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);
            
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }
            .map_err(|e| anyhow!("Failed to create descriptor pool: {}", e))?;

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&pipeline.descriptor_set_layout));
            
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }
            .map_err(|e| anyhow!("Failed to allocate descriptor sets: {}", e))?[0];

        let uniform_buffer = DynamicBuffer::new(
            allocator,
            std::mem::size_of::<LightUBO>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        )?;

        // Link the specific Uniform Buffer memory to the Descriptor Set
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(uniform_buffer.buffer)
            .offset(0)
            .range(std::mem::size_of::<LightUBO>() as u64);
            
        let write_set = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));
            
        unsafe { context.device.update_descriptor_sets(std::slice::from_ref(&write_set), &[]) };

        Ok(Self { 
            is_resized: false, egui_renderer, context, swapchain_mgr, sync,
            pipeline, vertex_buffer, index_buffer,
            uniform_buffer, descriptor_pool, descriptor_set
        })
    }

    pub fn draw_frame(
        &mut self, 
        window: &Window, 
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        pixels_per_point: f32,
        is_playing: bool,
        camera: &FreeformCamera,           
        visible_objects: &[GameObject],    
    ) -> Result<()> {
        
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut draw_calls = Vec::new();

        if is_playing {
            // Initialize our UBO payload with the ambient config
            let mut ubo = LightUBO {
                ambient_color: glam::Vec4::new(AMBIENT_LIGHT_COLOR[0], AMBIENT_LIGHT_COLOR[1], AMBIENT_LIGHT_COLOR[2], AMBIENT_LIGHT_INTENSITY),
                global_dir: glam::Vec4::ZERO,
                global_color: glam::Vec4::ZERO,
                camera_pos: glam::Vec4::new(camera.position.x, camera.position.y, camera.position.z, 0.0),
                spot_count: 0,
                _pad: [0; 3],
                spot_lights: [SpotLightData::default(); 10],
            };

            for obj in visible_objects {
                // Extract mesh data
                let vertex_offset = all_vertices.len() as i32;
                let index_start = all_indices.len() as u32;
                let index_count = obj.mesh.indices.len() as u32;

                all_vertices.extend_from_slice(&obj.mesh.vertices);
                all_indices.extend_from_slice(&obj.mesh.indices);

                draw_calls.push(DrawCall {
                    index_start,
                    index_count,
                    vertex_offset,
                    transform: obj.transform.get_model_matrix(),
                });

                // --- NEW: Gather active lights from the physical world ---
                if let Some(spot) = obj.spot_light {
                    if ubo.spot_count < 10 {
                        let idx = ubo.spot_count as usize;
                        ubo.spot_lights[idx] = SpotLightData {
                            // Link the light physically to the object's transform!
                            position: glam::Vec4::new(obj.transform.translation.x, obj.transform.translation.y, obj.transform.translation.z, spot.range),
                            direction: glam::Vec4::new(spot.direction.x, spot.direction.y, spot.direction.z, spot.intensity),
                            color: glam::Vec4::new(spot.color[0], spot.color[1], spot.color[2], spot.inner_cone_angle),
                            params: glam::Vec4::new(spot.outer_cone_angle, if spot.cast_shadows { 1.0 } else { 0.0 }, 0.0, 0.0),
                        };
                        ubo.spot_count += 1;
                    }
                }
            }

            let allocator = match self.context.allocator.as_ref() {
                Some(alloc) => alloc,
                None => return Err(anyhow!("Memory allocator missing during draw frame")),
            };

            self.vertex_buffer.upload_data(allocator, &all_vertices)?;
            self.index_buffer.upload_data(allocator, &all_indices)?;
            
            // Upload the entire lighting payload to the GPU!
            self.uniform_buffer.upload_data(allocator, &[ubo])?;
        }
        
        unsafe {
            self.context.device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX)
                .map_err(|e| anyhow!("Wait for fences failed: {}", e))?;

            if !textures_delta.set.is_empty() {
                self.egui_renderer.set_textures(
                    self.context.graphics_queue,
                    self.sync.command_pool,
                    textures_delta.set.as_slice(),
                ).map_err(|e| anyhow!("Failed to upload egui textures: {}", e))?;
            }

            if !textures_delta.free.is_empty() {
                self.egui_renderer.free_textures(textures_delta.free.as_slice())
                    .map_err(|e| anyhow!("Failed to free egui textures: {}", e))?;
            }

            let acquire_result = self.swapchain_mgr.loader.acquire_next_image(
                self.swapchain_mgr.swapchain, 
                u64::MAX, 
                self.sync.image_available, 
                vk::Fence::null()
            );
            
            let img_idx = match acquire_result {
                Ok((idx, is_suboptimal)) => {
                    if is_suboptimal || self.is_resized {
                        self.swapchain_mgr.recreate(&self.context, window)?;
                        self.is_resized = false;
                        return Ok(());
                    }
                    idx
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.swapchain_mgr.recreate(&self.context, window)?;
                    self.is_resized = false;
                    return Ok(());
                }
                Err(e) => return Err(anyhow!("Failed to acquire image: {}", e)),
            };

            self.context.device.reset_fences(&[self.sync.in_flight])
                .map_err(|e| anyhow!("Failed to reset fences: {}", e))?;
            self.context.device.reset_command_buffer(self.sync.command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| anyhow!("Failed to reset command buffer: {}", e))?;
                
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.context.device.begin_command_buffer(self.sync.command_buffer, &begin_info)
                .map_err(|e| anyhow!("Failed to begin command buffer: {}", e))?;
            
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

            self.context.device.cmd_begin_render_pass(
                self.sync.command_buffer, 
                &render_pass_info, 
                vk::SubpassContents::INLINE
            );

            if is_playing && !draw_calls.is_empty() {
                let viewport = vk::Viewport {
                    x: 0.0, y: 0.0,
                    width: self.swapchain_mgr.extent.width as f32,
                    height: self.swapchain_mgr.extent.height as f32,
                    min_depth: 0.0, max_depth: 1.0,
                };
                self.context.device.cmd_set_viewport(self.sync.command_buffer, 0, std::slice::from_ref(&viewport));

                let scissor = vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.swapchain_mgr.extent,
                };
                self.context.device.cmd_set_scissor(self.sync.command_buffer, 0, std::slice::from_ref(&scissor));

                self.context.device.cmd_bind_pipeline(self.sync.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.graphics_pipeline);
                self.context.device.cmd_bind_vertex_buffers(self.sync.command_buffer, 0, &[self.vertex_buffer.buffer], &[0]);
                self.context.device.cmd_bind_index_buffer(self.sync.command_buffer, self.index_buffer.buffer, 0, vk::IndexType::UINT32);

                // --- NEW: Bind the Descriptor Set for the Fragment Shader! ---
                self.context.device.cmd_bind_descriptor_sets(
                    self.sync.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.layout,
                    0,
                    std::slice::from_ref(&self.descriptor_set),
                    &[]
                );

                let aspect = self.swapchain_mgr.extent.width as f32 / self.swapchain_mgr.extent.height as f32;
                let mut proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);
                proj.y_axis.y *= -1.0; 
                
                let view = camera.get_view_matrix();
                let view_proj = proj * view;

                for call in draw_calls {
                    let pc = PushConstants {
                        view_proj,
                        model: call.transform,
                    };
                    
                    let pc_bytes = std::slice::from_raw_parts(
                        &pc as *const _ as *const u8,
                        std::mem::size_of::<PushConstants>(),
                    );

                    self.context.device.cmd_push_constants(
                        self.sync.command_buffer,
                        self.pipeline.layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        pc_bytes,
                    );

                    self.context.device.cmd_draw_indexed(
                        self.sync.command_buffer,
                        call.index_count,
                        1,
                        call.index_start,
                        call.vertex_offset,
                        0,
                    );
                }
            }

            if !clipped_primitives.is_empty() {
                self.egui_renderer.cmd_draw(
                    self.sync.command_buffer,
                    self.swapchain_mgr.extent,
                    pixels_per_point,
                    clipped_primitives,
                ).map_err(|e| anyhow!("Failed to draw egui primitives: {}", e))?;
            }
            
            self.context.device.cmd_end_render_pass(self.sync.command_buffer);
            self.context.device.end_command_buffer(self.sync.command_buffer)
                .map_err(|e| anyhow!("Failed to end command buffer: {}", e))?;

            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            self.context.device.queue_submit(
                self.context.graphics_queue, 
                std::slice::from_ref(&submit_info), 
                self.sync.in_flight
            ).map_err(|e| anyhow!("Failed to submit queue: {}", e))?;
            
            let active_swapchains = [self.swapchain_mgr.swapchain];
            let active_indices = [img_idx];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&active_swapchains)
                .image_indices(&active_indices);

            let present_result = self.swapchain_mgr.loader.queue_present(self.context.present_queue, &present_info);
            match present_result {
                Ok(is_suboptimal) => {
                    if is_suboptimal || self.is_resized {
                        self.swapchain_mgr.recreate(&self.context, window)?;
                        self.is_resized = false;
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.swapchain_mgr.recreate(&self.context, window)?;
                    self.is_resized = false;
                }
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
                self.uniform_buffer.destroy(allocator); // Drop the Uniform Buffer
            }
            
            self.context.device.destroy_descriptor_pool(self.descriptor_pool, None); // Drop the Pool
            self.pipeline.destroy(&self.context.device);
            self.sync.destroy(&self.context.device);
            self.swapchain_mgr.destroy(&self.context);
        }
    }
}