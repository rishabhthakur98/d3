// src/vulkan_logic/sync_objects.rs

use anyhow::{anyhow, Result};
use ash::vk;
use super::context::VulkanContext;

pub struct SyncObjects {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight: vk::Fence,
}

impl SyncObjects {
    pub fn new(context: &VulkanContext) -> Result<Self> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(context.graphics_queue_index);
            
        let command_pool = unsafe { context.device.create_command_pool(&pool_info, None) }
            .map_err(|e| anyhow!("Failed to create command pool: {}", e))?;
            
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
            
        let command_buffer = unsafe { context.device.allocate_command_buffers(&alloc_info) }
            .map_err(|e| anyhow!("Failed to allocate command buffer: {}", e))?[0];

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED); 

        let image_available = unsafe { context.device.create_semaphore(&semaphore_info, None) }
            .map_err(|e| anyhow!("Failed to create image_available semaphore: {}", e))?;
            
        let render_finished = unsafe { context.device.create_semaphore(&semaphore_info, None) }
            .map_err(|e| anyhow!("Failed to create render_finished semaphore: {}", e))?;
            
        let in_flight = unsafe { context.device.create_fence(&fence_info, None) }
            .map_err(|e| anyhow!("Failed to create in_flight fence: {}", e))?;

        Ok(Self { 
            command_pool, 
            command_buffer, 
            image_available, 
            render_finished, 
            in_flight 
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.in_flight, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}