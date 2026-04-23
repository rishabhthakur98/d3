// src/vulkan_logic/core/swapchain/framebuffers.rs

use anyhow::{anyhow, Result};
use ash::vk;
use crate::vulkan_logic::core::context::VulkanContext;

/// Zips the Render Pass, Image Views, and Depth Buffers together into the final canvases.
pub(crate) fn create_framebuffers(
    context: &VulkanContext,
    image_views: &[vk::ImageView],
    depth_image_view: vk::ImageView,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    
    image_views.iter().map(|&view| {
        let fb_attachments = [view, depth_image_view];
        let fb_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&fb_attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
            
        unsafe { context.device.create_framebuffer(&fb_info, None) }
    }).collect::<Result<Vec<_>, _>>().map_err(|e| anyhow!("Failed to create framebuffers: {}", e))
}