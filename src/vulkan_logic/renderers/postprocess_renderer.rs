// src/vulkan_logic/renderers/postprocess_renderer.rs
use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::memory::gpu_buffers::DynamicBuffer;
use crate::vulkan_logic::config::paths;

use crate::postprocessing::ssbo::PostProcessSSBO;
use crate::postprocessing::config::PostProcessConfig;
use crate::vulkan_logic::passes::offscreen_pass::OffscreenPass; 

pub struct PostProcessSystem {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub ssbo_buffer: DynamicBuffer,
}

impl PostProcessSystem {
    pub fn new(context: &VulkanContext, swapchain_render_pass: vk::RenderPass, offscreen: &OffscreenPass) -> Result<Self> {
        let allocator = context.allocator.as_ref().ok_or_else(|| anyhow!("Memory allocator not initialized"))?;

        // Initialize the SSBO buffer specifically mapped with Storage buffer usage flags
        let ssbo_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<PostProcessSSBO>(), vk::BufferUsageFlags::STORAGE_BUFFER)?;

        let ssbo_binding = vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let sampler_binding = vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        
        let layout_bindings = [ssbo_binding, sampler_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);
        
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }.map_err(|e| anyhow!("Failed to create descriptor layout: {}", e))?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }.map_err(|e| anyhow!("Failed to create pool: {}", e))?;

        let alloc_layouts = [descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&alloc_layouts);
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }.map_err(|e| anyhow!("Failed to allocate descriptors: {}", e))?[0];

        let mut system = Self { pipeline_layout: vk::PipelineLayout::null(), pipeline: vk::Pipeline::null(), descriptor_set_layout, descriptor_pool, descriptor_set, ssbo_buffer };
        
        system.update_descriptor(context, offscreen);

        let pipeline_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&pipeline_layouts);
        system.pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }.map_err(|e| anyhow!("Failed to create pipeline layout: {}", e))?;

        let vert_code = Self::read_shader(paths::POSTPROCESS_VERT)?;
        let frag_code = Self::read_shader(paths::POSTPROCESS_FRAG)?;
        let vert_mod = Self::create_module(&context.device, &vert_code)?;
        let frag_mod = Self::create_module(&context.device, &frag_code)?;
        let entry = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default(); 
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).cull_mode(vk::CullModeFlags::NONE).line_width(1.0);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false).depth_write_enable(false);
        let color_blend = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A);
        
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_mod).name(entry), 
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_mod).name(entry)
        ];
        
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachments = [color_blend];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages).vertex_input_state(&vertex_input).input_assembly_state(&assembly).viewport_state(&viewport).rasterization_state(&rasterizer)
            .multisample_state(&multisampling).depth_stencil_state(&depth_stencil).color_blend_state(&color_blending).dynamic_state(&dynamic_state_info)
            .layout(system.pipeline_layout).render_pass(swapchain_render_pass).subpass(0);

        system.pipeline = unsafe { context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None) }.map_err(|e| anyhow!("Postprocess pipeline creation failed: {:?}", e.1))?[0];

        unsafe { context.device.destroy_shader_module(vert_mod, None); context.device.destroy_shader_module(frag_mod, None); }
        Ok(system)
    }

    pub fn update_descriptor(&mut self, context: &VulkanContext, offscreen: &OffscreenPass) {
        let buffer_info = vk::DescriptorBufferInfo::default().buffer(self.ssbo_buffer.buffer).offset(0).range(std::mem::size_of::<PostProcessSSBO>() as u64);
        let image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(offscreen.color_view).sampler(offscreen.sampler);
        
        let buffer_infos = [buffer_info];
        let image_infos = [image_info];
        
        let writes = [
            vk::WriteDescriptorSet::default().dst_set(self.descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(&buffer_infos),
            vk::WriteDescriptorSet::default().dst_set(self.descriptor_set).dst_binding(1).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&image_infos),
        ];
        
        unsafe { context.device.update_descriptor_sets(&writes, &[]); }
    }

    pub fn draw(&mut self, context: &VulkanContext, cmd: vk::CommandBuffer, config: &PostProcessConfig, time: f32) -> Result<()> {
        let allocator = context.allocator.as_ref().ok_or_else(|| anyhow!("Memory allocator dropped"))?;
        
        let ssbo = PostProcessSSBO {
            enabled: if config.enabled { 1.0 } else { 0.0 },
            exposure: config.exposure,
            gamma: config.gamma,
            contrast: config.contrast,
            saturation: config.saturation,
            vignette_strength: config.vignette_strength,
            chromatic_aberration: config.chromatic_aberration,
            film_grain: config.film_grain,
            time,
            _pad: [0.0; 3],
        };
        
        self.ssbo_buffer.upload_data(allocator, &[ssbo])?;

        unsafe {
            context.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            let descriptor_sets = [self.descriptor_set];
            context.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &descriptor_sets, &[]);
            context.device.cmd_draw(cmd, 3, 1, 0, 0); 
        }
        Ok(())
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            if let Some(a) = context.allocator.as_ref() { self.ssbo_buffer.destroy(a); }
            context.device.destroy_descriptor_pool(self.descriptor_pool, None);
            context.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            context.device.destroy_pipeline(self.pipeline, None);
            context.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
    
    fn read_shader(p: &str) -> Result<Vec<u8>> { 
        let mut f = File::open(p).map_err(|e| anyhow!("Failed to open shader {}: {}", p, e))?; 
        let mut b = Vec::new(); 
        f.read_to_end(&mut b).map_err(|e| anyhow!("Failed to read shader {}: {}", p, e))?; 
        Ok(b) 
    }
    fn create_module(d: &ash::Device, c: &[u8]) -> Result<vk::ShaderModule> { 
        let (p, code, s) = unsafe { c.align_to::<u32>() }; 
        if !p.is_empty() || !s.is_empty() { return Err(anyhow!("Shader alignment issue")); }
        unsafe { d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None) }.map_err(|e| anyhow!("Module creation error: {}", e)) 
    }
}