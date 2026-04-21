use anyhow::{anyhow, Result};
use ash::{khr::swapchain, vk};
use winit::window::Window;
use super::context::VulkanContext;
use vk_mem::{Allocation, AllocationCreateInfo, MemoryUsage, Alloc}; // NEW

pub struct SwapchainManager {
    pub loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub render_pass: vk::RenderPass,
    pub framebuffers: Vec<vk::Framebuffer>,
    
    // NEW: Depth Buffer Resources
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

        // --- NEW: CREATING THE Z-BUFFER (DEPTH BUFFER) ---
        let depth_format = vk::Format::D32_SFLOAT;
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
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

        let depth_image_view = unsafe { context.device.create_image_view(&depth_view_info, None) }
            .map_err(|e| anyhow!("Failed to create depth image view: {}", e))?;

        // --- RENDER PASS UPDATE (Added Depth Attachment) ---
        let color_attachment = vk::AttachmentDescription::default()
            .format(format.format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR).store_op(vk::AttachmentStoreOp::STORE) 
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        
        let depth_attachment = vk::AttachmentDescription::default()
            .format(depth_format).samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR) // Clear Z-Buffer every frame
            .store_op(vk::AttachmentStoreOp::DONT_CARE) // We don't need to read it later
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE).stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED).final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_ref = vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_ref = vk::AttachmentReference::default().attachment(1).layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref))
            .depth_stencil_attachment(&depth_ref);
            
        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL).dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

        let attachments = [color_attachment, depth_attachment];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        let render_pass = unsafe { context.device.create_render_pass(&render_pass_info, None) }.map_err(|e| anyhow!("{}", e))?;

        // Framebuffers now combine color AND depth views
        let framebuffers = image_views.iter().map(|&view| {
            let fb_attachments = [view, depth_image_view];
            let fb_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&fb_attachments)
                .width(extent.width).height(extent.height).layers(1);
            unsafe { context.device.create_framebuffer(&fb_info, None) }
        }).collect::<Result<Vec<_>, _>>().map_err(|e| anyhow!("{}", e))?;

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

            // Cleanup old resources
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

            // Rebuild Depth Buffer
            let depth_format = vk::Format::D32_SFLOAT;
            let depth_image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D).extent(vk::Extent3D { width: extent.width, height: extent.height, depth: 1 })
                .mip_levels(1).array_layers(1).format(depth_format).tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED).usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .samples(vk::SampleCountFlags::TYPE_1).sharing_mode(vk::SharingMode::EXCLUSIVE);

            let depth_alloc_info = AllocationCreateInfo { usage: MemoryUsage::Auto, ..Default::default() };
            let (depth_image, depth_allocation) = allocator.create_image(&depth_image_info, &depth_alloc_info).map_err(|e| anyhow!("{}", e))?;

            let depth_view_info = vk::ImageViewCreateInfo::default()
                .image(depth_image).view_type(vk::ImageViewType::TYPE_2D).format(depth_format)
                .subresource_range(vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::DEPTH, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 });

            self.depth_image_view = context.device.create_image_view(&depth_view_info, None).map_err(|e| anyhow!("{}", e))?;
            self.depth_image = depth_image;
            self.depth_allocation = depth_allocation;

            self.framebuffers = self.image_views.iter().map(|&view| {
                let fb_attachments = [view, self.depth_image_view];
                let fb_info = vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass).attachments(&fb_attachments)
                    .width(extent.width).height(extent.height).layers(1);
                context.device.create_framebuffer(&fb_info, None)
            }).collect::<Result<Vec<_>, _>>().map_err(|e| anyhow!("{}", e))?;
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