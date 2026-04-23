// src/vulkan_logic/core/swapchain/manager.rs

use anyhow::{anyhow, Result};
use ash::{khr::swapchain, vk};
use winit::window::Window;

// FIXED: Removed the unused `vk_mem::Allocation` warning
use vk_mem::Allocation; 

use crate::vulkan_logic::core::context::VulkanContext;
use super::depth::create_depth_buffer;
use super::render_pass::create_render_pass;
use super::framebuffers::create_framebuffers;

pub struct SwapchainManager {
    pub loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    
    pub depth_image: vk::Image,
    pub depth_allocation: Allocation,
    pub depth_image_view: vk::ImageView,
}

impl SwapchainManager {
    pub fn new(context: &VulkanContext, window: &Window) -> Result<Self> {
        let allocator = match context.allocator.as_ref() {
            Some(a) => a,
            None => return Err(anyhow!("Allocator missing")),
        };

        let caps = unsafe { context.surface_loader.get_physical_device_surface_capabilities(context.physical_device, context.surface) }
            .map_err(|e| anyhow!("Failed to get capabilities: {}", e))?;
        
        let formats = unsafe { context.surface_loader.get_physical_device_surface_formats(context.physical_device, context.surface) }
            .map_err(|e| anyhow!("Failed to get formats: {}", e))?;
        
        let format = formats[0]; 
        let extent = vk::Extent2D { width: window.inner_size().width, height: window.inner_size().height };
        
        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count { image_count = caps.max_image_count; }

        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(context.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO) 
            .clipped(true);

        let indices = [context.graphics_queue_index, context.present_queue_index];
        if context.graphics_queue_index != context.present_queue_index {
            create_info = create_info.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&indices);
        }

        let loader = swapchain::Device::new(&context.instance, &context.device);
        let swapchain = unsafe { loader.create_swapchain(&create_info, None) }.map_err(|e| anyhow!("{}", e))?;
        let images = unsafe { loader.get_swapchain_images(swapchain) }.map_err(|e| anyhow!("{}", e))?;

        let image_views = images.iter().map(|&img| {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(img).view_type(vk::ImageViewType::TYPE_2D).format(format.format)
                .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
            unsafe { context.device.create_image_view(&view_info, None) }
        }).collect::<Result<Vec<_>, _>>().map_err(|e| anyhow!("{}", e))?;

        let depth_format = vk::Format::D32_SFLOAT;
        let (depth_image, depth_allocation, depth_image_view) = create_depth_buffer(context, allocator, extent, depth_format)?;
        let render_pass = create_render_pass(context, format.format, depth_format)?;
        let framebuffers = create_framebuffers(context, &image_views, depth_image_view, render_pass, extent)?;

        Ok(Self { 
            loader, swapchain, images, image_views, extent, render_pass, framebuffers, 
            depth_image, depth_allocation, depth_image_view 
        })
    }

    pub fn recreate(&mut self, context: &VulkanContext, window: &Window) -> Result<()> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 { return Ok(()); } 
        let allocator = match context.allocator.as_ref() {
            Some(a) => a,
            None => return Err(anyhow!("Allocator missing")),
        };

        unsafe {
            context.device.device_wait_idle().map_err(|e| anyhow!("{}", e))?;

            for &fb in &self.framebuffers { context.device.destroy_framebuffer(fb, None); }
            for &view in &self.image_views { context.device.destroy_image_view(view, None); }
            context.device.destroy_image_view(self.depth_image_view, None);
            allocator.destroy_image(self.depth_image, &mut self.depth_allocation);
            self.loader.destroy_swapchain(self.swapchain, None);

            let caps = context.surface_loader.get_physical_device_surface_capabilities(context.physical_device, context.surface).map_err(|e| anyhow!("{}", e))?;
            let formats = context.surface_loader.get_physical_device_surface_formats(context.physical_device, context.surface).map_err(|e| anyhow!("{}", e))?;
            let format = formats[0];

            let extent = vk::Extent2D {
                width: size.width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: size.height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            };

            let mut image_count = caps.min_image_count + 1;
            if caps.max_image_count > 0 && image_count > caps.max_image_count { image_count = caps.max_image_count; }

            let mut create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(context.surface).min_image_count(image_count).image_format(format.format)
                .image_color_space(format.color_space).image_extent(extent).image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT).image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(caps.current_transform).composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO).clipped(true);

            let indices = [context.graphics_queue_index, context.present_queue_index];
            if context.graphics_queue_index != context.present_queue_index {
                create_info = create_info.image_sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&indices);
            }

            self.swapchain = self.loader.create_swapchain(&create_info, None).map_err(|e| anyhow!("{}", e))?;
            self.images = self.loader.get_swapchain_images(self.swapchain).map_err(|e| anyhow!("{}", e))?;
            self.extent = extent;

            self.image_views = self.images.iter().map(|&img| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(img).view_type(vk::ImageViewType::TYPE_2D).format(format.format)
                    .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
                context.device.create_image_view(&view_info, None)
            }).collect::<Result<Vec<_>, _>>().map_err(|e| anyhow!("{}", e))?;

            let depth_format = vk::Format::D32_SFLOAT;
            let (depth_image, depth_allocation, depth_image_view) = create_depth_buffer(context, allocator, extent, depth_format)?;
            
            self.depth_image = depth_image;
            self.depth_allocation = depth_allocation;
            self.depth_image_view = depth_image_view;

            self.framebuffers = create_framebuffers(context, &self.image_views, self.depth_image_view, self.render_pass, extent)?;
        }
        Ok(())
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            for &fb in &self.framebuffers { context.device.destroy_framebuffer(fb, None); }
            context.device.destroy_render_pass(self.render_pass, None);
            for &view in &self.image_views { context.device.destroy_image_view(view, None); }
            context.device.destroy_image_view(self.depth_image_view, None);
            
            if let Some(alloc) = context.allocator.as_ref() {
                alloc.destroy_image(self.depth_image, &mut self.depth_allocation);
            }
            
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}