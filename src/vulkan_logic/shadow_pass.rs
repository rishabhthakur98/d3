// src/vulkan_logic/shadow_pass.rs

use anyhow::{anyhow, Result};
use ash::vk;
use vk_mem::{Allocation, AllocationCreateInfo, MemoryUsage, Alloc};
use super::context::VulkanContext;

pub const SHADOW_MAP_DIM: u32 = 2048; // 2K resolution for crisp shadows

pub struct ShadowPass {
    pub render_pass: vk::RenderPass,
    pub depth_image: vk::Image,
    pub depth_alloc: Allocation,
    pub depth_view: vk::ImageView,
    pub framebuffer: vk::Framebuffer,
    pub sampler: vk::Sampler,
}

impl ShadowPass {
    pub fn new(context: &VulkanContext) -> Result<Self> {
        let allocator = match context.allocator.as_ref() {
            Some(a) => a,
            None => return Err(anyhow!("Allocator missing")),
        };

        let depth_format = vk::Format::D32_SFLOAT;

        // 1. Create the invisible Depth Image
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width: SHADOW_MAP_DIM, height: SHADOW_MAP_DIM, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(depth_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            // SAMPLED flag allows us to pass this image to the main Fragment Shader later!
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto,
            ..Default::default()
        };

        let (depth_image, depth_alloc) = unsafe { allocator.create_image(&image_info, &alloc_info) }
            .map_err(|e| anyhow!("Failed to create shadow map image: {}", e))?;

        let view_info = vk::ImageViewCreateInfo::default()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1
            });

        let depth_view = unsafe { context.device.create_image_view(&view_info, None) }
            .map_err(|e| anyhow!("Failed to create shadow map view: {}", e))?;

        // 2. Create the Custom Render Pass
        let attachment = vk::AttachmentDescription::default()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE) // IMPORTANT: We must STORE it to read it later!
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            // Final layout is READ_ONLY so the fragment shader can sample it as a texture
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL); 

        let attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .depth_stencil_attachment(&attachment_ref);

        let dependencies = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE),
            vk::SubpassDependency::default()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ),
        ];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let render_pass = unsafe { context.device.create_render_pass(&render_pass_info, None) }
            .map_err(|e| anyhow!("Failed to create shadow render pass: {}", e))?;

        // 3. Create the off-screen Framebuffer
        let fb_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(std::slice::from_ref(&depth_view))
            .width(SHADOW_MAP_DIM)
            .height(SHADOW_MAP_DIM)
            .layers(1);

        let framebuffer = unsafe { context.device.create_framebuffer(&fb_info, None) }
            .map_err(|e| anyhow!("Failed to create shadow framebuffer: {}", e))?;

        // 4. Create the Sampler (Defines how the GPU scales the texture if it's stretched)
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .max_anisotropy(1.0)
            .min_lod(0.0)
            .max_lod(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);

        let sampler = unsafe { context.device.create_sampler(&sampler_info, None) }
            .map_err(|e| anyhow!("Failed to create shadow sampler: {}", e))?;

        Ok(Self { render_pass, depth_image, depth_alloc, depth_view, framebuffer, sampler })
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            context.device.destroy_sampler(self.sampler, None);
            context.device.destroy_framebuffer(self.framebuffer, None);
            context.device.destroy_image_view(self.depth_view, None);
            context.device.destroy_render_pass(self.render_pass, None);
            if let Some(alloc) = context.allocator.as_ref() {
                alloc.destroy_image(self.depth_image, &mut self.depth_alloc);
            }
        }
    }
}