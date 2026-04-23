// src/postprocessing/offscreen.rs
use anyhow::{anyhow, Result};
use ash::vk;

// FIXED: Brought the `Alloc` trait into scope and removed unused imports
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, MemoryUsage}; 
use crate::vulkan_logic::core::context::VulkanContext;

/// Intercepts the rendering pipeline into a massive High Dynamic Range (HDR) floating-point buffer.
pub struct OffscreenPass {
    pub render_pass: vk::RenderPass,
    pub color_image: vk::Image,
    pub color_alloc: Allocation,
    pub color_view: vk::ImageView,
    pub depth_image: vk::Image,
    pub depth_alloc: Allocation,
    pub depth_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub framebuffer: vk::Framebuffer,
}

impl OffscreenPass {
    pub fn new(context: &VulkanContext, extent: vk::Extent2D) -> Result<Self> {
        let allocator = match context.allocator.as_ref() {
            Some(a) => a,
            None => return Err(anyhow!("Allocator missing")),
        };
        
        // AAA Graphics require HDR precision buffers (16-bit float per channel!)
        let color_format = vk::Format::R16G16B16A16_SFLOAT;
        let depth_format = vk::Format::D32_SFLOAT;

        // 1. Create HDR Color Attachment
        let color_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D).extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 })
            .mip_levels(1).array_layers(1).format(color_format).tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED).usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .samples(vk::SampleCountFlags::TYPE_1).sharing_mode(vk::SharingMode::EXCLUSIVE);
            
        let (color_image, color_alloc) = unsafe { allocator.create_image(&color_info, &AllocationCreateInfo { usage: MemoryUsage::Auto, ..Default::default() }) }.map_err(|e| anyhow!("{}", e))?;

        let color_view_info = vk::ImageViewCreateInfo::default().image(color_image).view_type(vk::ImageViewType::TYPE_2D).format(color_format)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
        let color_view = unsafe { context.device.create_image_view(&color_view_info, None) }.map_err(|e| anyhow!("{}", e))?;

        // 2. Create standard Z-Buffer
        let depth_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D).extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 })
            .mip_levels(1).array_layers(1).format(depth_format).tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED).usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlags::TYPE_1).sharing_mode(vk::SharingMode::EXCLUSIVE);
            
        let (depth_image, depth_alloc) = unsafe { allocator.create_image(&depth_info, &AllocationCreateInfo { usage: MemoryUsage::Auto, ..Default::default() }) }.map_err(|e| anyhow!("{}", e))?;

        let depth_view_info = vk::ImageViewCreateInfo::default().image(depth_image).view_type(vk::ImageViewType::TYPE_2D).format(depth_format)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });
        let depth_view = unsafe { context.device.create_image_view(&depth_view_info, None) }.map_err(|e| anyhow!("{}", e))?;

        // 3. Create isolated Render Pass
        let color_attachment = vk::AttachmentDescription::default()
            .format(color_format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL); 
            
        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_ref = vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_ref = vk::AttachmentReference::default().attachment(1).layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        
        let subpass = vk::SubpassDescription::default().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref)).depth_stencil_attachment(&depth_ref);

        let dep1 = vk::SubpassDependency::default().src_subpass(vk::SUBPASS_EXTERNAL).dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER).dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::SHADER_READ).dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
            
        let dep2 = vk::SubpassDependency::default().src_subpass(0).dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT).dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE).dst_access_mask(vk::AccessFlags::SHADER_READ);

        let attachments = [color_attachment, depth_attachment];
        let deps = [dep1, dep2];
        let rp_info = vk::RenderPassCreateInfo::default().attachments(&attachments).subpasses(std::slice::from_ref(&subpass)).dependencies(&deps);
        let render_pass = unsafe { context.device.create_render_pass(&rp_info, None) }.map_err(|e| anyhow!("{}", e))?;

        // 4. Framebuffer & Sampler
        let fb_attachments = [color_view, depth_view];
        let fb_info = vk::FramebufferCreateInfo::default().render_pass(render_pass).attachments(&fb_attachments).width(extent.width).height(extent.height).layers(1);
        let framebuffer = unsafe { context.device.create_framebuffer(&fb_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let sampler_info = vk::SamplerCreateInfo::default().mag_filter(vk::Filter::LINEAR).min_filter(vk::Filter::LINEAR).address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE).address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
        let sampler = unsafe { context.device.create_sampler(&sampler_info, None) }.map_err(|e| anyhow!("{}", e))?;

        Ok(Self { render_pass, color_image, color_alloc, color_view, depth_image, depth_alloc, depth_view, sampler, framebuffer })
    }

    pub fn recreate(&mut self, context: &VulkanContext, extent: vk::Extent2D) -> Result<()> {
        self.destroy(context);
        *self = Self::new(context, extent)?;
        Ok(())
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            let device = &context.device;
            if let Some(alloc) = context.allocator.as_ref() {
                device.destroy_sampler(self.sampler, None);
                device.destroy_framebuffer(self.framebuffer, None);
                device.destroy_render_pass(self.render_pass, None);
                device.destroy_image_view(self.color_view, None);
                device.destroy_image_view(self.depth_view, None);
                alloc.destroy_image(self.color_image, &mut self.color_alloc);
                alloc.destroy_image(self.depth_image, &mut self.depth_alloc);
            }
        }
    }
}