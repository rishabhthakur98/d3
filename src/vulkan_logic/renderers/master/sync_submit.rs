// src/vulkan_logic/renderers/master/sync_submit.rs

use anyhow::{anyhow, Result};
use ash::vk;
use winit::window::Window;
use super::renderer::MasterRenderer;

impl MasterRenderer {
    /// Ensure the GPU finishes the last designated task before we touch any resources
    pub(crate) fn wait_fences(&self) -> Result<()> {
        unsafe {
            self.context.device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX)
                .map_err(|e| anyhow!("Wait for fences failed: {}", e))?;
        }
        Ok(())
    }

    /// Pipes Egui texture changes into Vulkan space before rendering UI elements
    pub(crate) fn update_egui_textures(&mut self, textures_delta: &egui::TexturesDelta) -> Result<()> {
        unsafe {
            if !textures_delta.set.is_empty() {
                self.egui_renderer.set_textures(self.context.graphics_queue, self.sync.command_pool, textures_delta.set.as_slice())
                    .map_err(|e| anyhow!("Failed to upload egui textures: {}", e))?;
            }
            if !textures_delta.free.is_empty() {
                self.egui_renderer.free_textures(textures_delta.free.as_slice())
                    .map_err(|e| anyhow!("Failed to free egui textures: {}", e))?;
            }
        }
        Ok(())
    }

    /// Intercepts the next image on the swapchain chain or correctly reconstructs the swapchain if scaled
    pub(crate) fn acquire_image(&mut self, window: &Window) -> Result<Option<u32>> {
        let acquire_result = unsafe {
            self.swapchain_mgr.loader.acquire_next_image(self.swapchain_mgr.swapchain, u64::MAX, self.sync.image_available, vk::Fence::null())
        };

        match acquire_result {
            Ok((idx, is_suboptimal)) => {
                if is_suboptimal || self.is_resized { 
                    self.swapchain_mgr.recreate(&self.context, window)?; 
                    self.is_resized = false; 
                    return Ok(None); 
                }
                Ok(Some(idx))
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { 
                self.swapchain_mgr.recreate(&self.context, window)?; 
                self.is_resized = false; 
                Ok(None) 
            }
            Err(e) => Err(anyhow!("Failed to acquire image: {}", e)),
        }
    }

    /// Prepares the master command buffer to write the rendering queues
    pub(crate) fn begin_command_buffer(&self) -> Result<()> {
        unsafe {
            self.context.device.reset_fences(&[self.sync.in_flight])
                .map_err(|e| anyhow!("Failed to reset fences: {}", e))?;
            self.context.device.reset_command_buffer(self.sync.command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| anyhow!("Failed to reset command buffer: {}", e))?;
                
            let begin_info = vk::CommandBufferBeginInfo::default();
            self.context.device.begin_command_buffer(self.sync.command_buffer, &begin_info)
                .map_err(|e| anyhow!("Failed to begin command buffer: {}", e))?;
        }
        Ok(())
    }

    /// Signals the end of writing to the command buffer
    pub(crate) fn end_command_buffer(&self) -> Result<()> {
        unsafe {
            self.context.device.end_command_buffer(self.sync.command_buffer)
                .map_err(|e| anyhow!("Failed to end command buffer: {}", e))?;
        }
        Ok(())
    }

    /// Final stage: Fire off the command buffers to the actual GPU hardware queue
    pub(crate) fn submit_and_present(&mut self, img_idx: u32, window: &Window) -> Result<()> {
        unsafe {
            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
                
            self.context.device.queue_submit(self.context.graphics_queue, std::slice::from_ref(&submit_info), self.sync.in_flight)
                .map_err(|e| anyhow!("Failed to submit queue: {}", e))?;
            
            let active_swapchains = [self.swapchain_mgr.swapchain];
            let active_indices = [img_idx];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&active_swapchains)
                .image_indices(&active_indices);

            // Command successful execution visually to the screen
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