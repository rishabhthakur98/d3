// src/vulkan_logic/core/swapchain/depth.rs

use anyhow::{anyhow, Result};
use ash::vk;
use vk_mem::{Allocation, AllocationCreateInfo, MemoryUsage, Alloc, Allocator};
use crate::vulkan_logic::core::context::VulkanContext;

/// Strictly handles the memory allocation and view creation for the Z-Buffer (Depth).
pub(crate) fn create_depth_buffer(
    context: &VulkanContext,
    allocator: &Allocator,
    extent: vk::Extent2D,
    depth_format: vk::Format,
) -> Result<(vk::Image, Allocation, vk::ImageView)> {
    
    let depth_image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .format(depth_format)
        .tiling(vk::ImageTiling::OPTIMAL)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let depth_alloc_info = AllocationCreateInfo {
        usage: MemoryUsage::Auto,
        ..Default::default()
    };

    let (depth_image, depth_allocation) = unsafe { allocator.create_image(&depth_image_info, &depth_alloc_info) }
        .map_err(|e| anyhow!("Failed to allocate depth buffer: {}", e))?;

    let depth_view_info = vk::ImageViewCreateInfo::default()
        .image(depth_image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(depth_format)
        .subresource_range(vk::ImageSubresourceRange { 
            aspect_mask: vk::ImageAspectFlags::DEPTH, 
            base_mip_level: 0, 
            level_count: 1, 
            base_array_layer: 0, 
            layer_count: 1 
        });

    let depth_image_view = unsafe { context.device.create_image_view(&depth_view_info, None) }
        .map_err(|e| anyhow!("Failed to create depth image view: {}", e))?;

    Ok((depth_image, depth_allocation, depth_image_view))
}