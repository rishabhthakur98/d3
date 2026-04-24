// src/vulkan_logic/renderers/master/sync_submit.rs
use anyhow::{anyhow, Result};
use ash::vk;
use winit::window::Window;
use super::renderer::MasterRenderer;

impl MasterRenderer {
    pub(crate) fn wait_fences(&self) -> Result<()> {
        unsafe { 
            self.context.device.wait_for_fences(&[self.sync.in_flight], true, u64::MAX)
                .map_err(|e| anyhow!("GPU synchronization fence timeout detected: {}", e))?; 
        }
        Ok(())
    }

    pub(crate) fn update_egui_textures(&mut self, textures_delta: &egui::TexturesDelta) -> Result<()> {
        if !textures_delta.set.is_empty() { 
            self.egui_renderer.set_textures(self.context.graphics_queue, self.sync.command_pool, textures_delta.set.as_slice())
                .map_err(|e| anyhow!("Failed to securely mount Egui dynamic textures: {}", e))?; 
        }
        if !textures_delta.free.is_empty() { 
            self.egui_renderer.free_textures(textures_delta.free.as_slice())
                .map_err(|e| anyhow!("Failed to release Egui textures: {}", e))?; 
        }
        Ok(())
    }

    pub(crate) fn acquire_image(&mut self, window: &Window) -> Result<Option<u32>> {
        let acquire_result = unsafe { 
            self.swapchain_mgr.loader.acquire_next_image(self.swapchain_mgr.swapchain, u64::MAX, self.sync.image_available, vk::Fence::null()) 
        };

        match acquire_result {
            Ok((idx, is_suboptimal)) => {
                if is_suboptimal || self.is_resized { 
                    self.swapchain_mgr.recreate(&self.context, window)?; 
                    self.offscreen_pass.recreate(&self.context, self.swapchain_mgr.extent)?;
                    self.post_process_system.update_descriptor(&self.context, &self.offscreen_pass);
                    self.is_resized = false; 
                    return Ok(None); 
                }
                Ok(Some(idx))
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { 
                self.swapchain_mgr.recreate(&self.context, window)?; 
                self.offscreen_pass.recreate(&self.context, self.swapchain_mgr.extent)?;
                self.post_process_system.update_descriptor(&self.context, &self.offscreen_pass);
                self.is_resized = false; 
                Ok(None) 
            }
            Err(e) => Err(anyhow!("Critical swapchain image acquisition failure: {}", e)),
        }
    }

    pub(crate) fn begin_command_buffer(&self) -> Result<()> {
        unsafe {
            self.context.device.reset_fences(&[self.sync.in_flight])
                .map_err(|e| anyhow!("Failed to reset CPU-GPU execution fences: {}", e))?;
                
            self.context.device.reset_command_buffer(self.sync.command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| anyhow!("Command buffer state clearance failed: {}", e))?;
                
            self.context.device.begin_command_buffer(self.sync.command_buffer, &vk::CommandBufferBeginInfo::default())
                .map_err(|e| anyhow!("Initialization of command buffer execution failed: {}", e))?;
        }
        Ok(())
    }

    pub(crate) fn end_command_buffer(&self) -> Result<()> {
        unsafe { 
            self.context.device.end_command_buffer(self.sync.command_buffer)
                .map_err(|e| anyhow!("Failed finalizing command buffer recording: {}", e))?; 
        }
        Ok(())
    }

    pub(crate) fn submit_and_present(&mut self, img_idx: u32, window: &Window) -> Result<()> {
        unsafe {
            let wait_semaphores = [self.sync.image_available];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [self.sync.command_buffer];
            let signal_semaphores = [self.sync.render_finished];

            let submit_info = vk::SubmitInfo::default().wait_semaphores(&wait_semaphores).wait_dst_stage_mask(&wait_stages).command_buffers(&command_buffers).signal_semaphores(&signal_semaphores);
            self.context.device.queue_submit(self.context.graphics_queue, std::slice::from_ref(&submit_info), self.sync.in_flight)
                .map_err(|e| anyhow!("Device submission blocked or failed: {}", e))?;
            
            let active_swapchains = [self.swapchain_mgr.swapchain];
            let active_indices = [img_idx];
            let present_info = vk::PresentInfoKHR::default().wait_semaphores(&signal_semaphores).swapchains(&active_swapchains).image_indices(&active_indices);

            match self.swapchain_mgr.loader.queue_present(self.context.present_queue, &present_info) {
                Ok(is_suboptimal) => { 
                    if is_suboptimal || self.is_resized { 
                        self.swapchain_mgr.recreate(&self.context, window)?; 
                        self.offscreen_pass.recreate(&self.context, self.swapchain_mgr.extent)?;
                        self.post_process_system.update_descriptor(&self.context, &self.offscreen_pass);
                        self.is_resized = false; 
                    } 
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { 
                    self.swapchain_mgr.recreate(&self.context, window)?; 
                    self.offscreen_pass.recreate(&self.context, self.swapchain_mgr.extent)?;
                    self.post_process_system.update_descriptor(&self.context, &self.offscreen_pass);
                    self.is_resized = false; 
                }
                Err(e) => return Err(anyhow!("Surface presentation error indicating detached OS canvas: {}", e)),
            }
        }
        Ok(())
    }
}