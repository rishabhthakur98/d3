// src/vulkan_logic/swapchain_manager.rs

use anyhow::{anyhow, Result};
use ash::{khr::swapchain, vk};
use winit::window::Window;
use super::context::VulkanContext;

pub struct SwapchainManager {
    pub loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl SwapchainManager {
    pub fn new(context: &VulkanContext, window: &Window) -> Result<Self> {
        let caps = unsafe { 
            context.surface_loader.get_physical_device_surface_capabilities(context.physical_device, context.surface) 
        }.map_err(|e| anyhow!("Failed to get capabilities: {}", e))?;
        
        let formats = unsafe { 
            context.surface_loader.get_physical_device_surface_formats(context.physical_device, context.surface) 
        }.map_err(|e| anyhow!("Failed to get formats: {}", e))?;
        
        let format = formats[0]; 
        
        let extent = vk::Extent2D { 
            width: window.inner_size().width, 
            height: window.inner_size().height 
        };
        
        let mut image_count = caps.min_image_count + 1;
        if caps.max_image_count > 0 && image_count > caps.max_image_count { 
            image_count = caps.max_image_count; 
        }

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
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&indices);
        }

        let loader = swapchain::Device::new(&context.instance, &context.device);
        let swapchain = unsafe { loader.create_swapchain(&create_info, None) }
            .map_err(|e| anyhow!("Failed to create swapchain: {}", e))?;
            
        let images = unsafe { loader.get_swapchain_images(swapchain) }
            .map_err(|e| anyhow!("Failed to get swapchain images: {}", e))?;

        let image_views = images.iter().map(|&img| {
            let view_info = vk::ImageViewCreateInfo::default()
                .image(img)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(vk::ImageSubresourceRange { 
                    aspect_mask: vk::ImageAspectFlags::COLOR, 
                    base_mip_level: 0, 
                    level_count: 1, 
                    base_array_layer: 0, 
                    layer_count: 1 
                });
            unsafe { context.device.create_image_view(&view_info, None) }
        }).collect::<Result<Vec<_>, _>>()
          .map_err(|e| anyhow!("Failed to create image views: {}", e))?;

        let color_attachment = vk::AttachmentDescription::default()
            .format(format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)   
            .store_op(vk::AttachmentStoreOp::STORE) 
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        
        let color_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref));
            
        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe { context.device.create_render_pass(&render_pass_info, None) }
            .map_err(|e| anyhow!("Failed to create render pass: {}", e))?;

        let framebuffers = image_views.iter().map(|&view| {
            let fb_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(std::slice::from_ref(&view))
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe { context.device.create_framebuffer(&fb_info, None) }
        }).collect::<Result<Vec<_>, _>>()
          .map_err(|e| anyhow!("Failed to create framebuffers: {}", e))?;

        Ok(Self { loader, swapchain, images, image_views, extent, render_pass, framebuffers })
    }

    pub fn recreate(&mut self, context: &VulkanContext, window: &Window) -> Result<()> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 { return Ok(()); } 

        unsafe {
            context.device.device_wait_idle()
                .map_err(|e| anyhow!("Failed to wait for device idle: {}", e))?;

            for &fb in &self.framebuffers { context.device.destroy_framebuffer(fb, None); }
            for &view in &self.image_views { context.device.destroy_image_view(view, None); }
            self.loader.destroy_swapchain(self.swapchain, None);

            let caps = context.surface_loader.get_physical_device_surface_capabilities(context.physical_device, context.surface)
                .map_err(|e| anyhow!("Failed getting caps: {}", e))?;
            let formats = context.surface_loader.get_physical_device_surface_formats(context.physical_device, context.surface)
                .map_err(|e| anyhow!("Failed getting formats: {}", e))?;
            let format = formats[0];

            let extent = vk::Extent2D {
                width: size.width.clamp(caps.min_image_extent.width, caps.max_image_extent.width),
                height: size.height.clamp(caps.min_image_extent.height, caps.max_image_extent.height),
            };

            let mut image_count = caps.min_image_count + 1;
            if caps.max_image_count > 0 && image_count > caps.max_image_count { 
                image_count = caps.max_image_count; 
            }

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
                create_info = create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&indices);
            }

            self.swapchain = self.loader.create_swapchain(&create_info, None)
                .map_err(|e| anyhow!("Failed recreating swapchain: {}", e))?;
            self.images = self.loader.get_swapchain_images(self.swapchain)
                .map_err(|e| anyhow!("Failed getting recreated swapchain images: {}", e))?;
            self.extent = extent;

            self.image_views = self.images.iter().map(|&img| {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(img)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .subresource_range(vk::ImageSubresourceRange { 
                        aspect_mask: vk::ImageAspectFlags::COLOR, 
                        base_mip_level: 0, 
                        level_count: 1, 
                        base_array_layer: 0, 
                        layer_count: 1 
                    });
                context.device.create_image_view(&view_info, None)
            }).collect::<Result<Vec<_>, _>>()
              .map_err(|e| anyhow!("Failed to create new image views: {}", e))?;

            self.framebuffers = self.image_views.iter().map(|&view| {
                let fb_info = vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass)
                    .attachments(std::slice::from_ref(&view))
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                context.device.create_framebuffer(&fb_info, None)
            }).collect::<Result<Vec<_>, _>>()
              .map_err(|e| anyhow!("Failed to create new framebuffers: {}", e))?;
        }
        Ok(())
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            for &fb in &self.framebuffers { device.destroy_framebuffer(fb, None); }
            device.destroy_render_pass(self.render_pass, None);
            for &view in &self.image_views { device.destroy_image_view(view, None); }
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}