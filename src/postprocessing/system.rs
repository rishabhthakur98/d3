// src/postprocessing/system.rs

use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::memory::gpu_buffers::DynamicBuffer;
use super::ubo::PostProcessUBO;
use super::config::PostProcessConfig;
use super::offscreen::OffscreenPass;

pub struct PostProcessSystem {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub ubo_buffer: DynamicBuffer,
}

impl PostProcessSystem {
    pub fn new(context: &VulkanContext, swapchain_render_pass: vk::RenderPass, offscreen: &OffscreenPass) -> Result<Self> {
        let allocator = context.allocator.as_ref().unwrap();

        let ubo_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<PostProcessUBO>(), vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let ubo_binding = vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let sampler_binding = vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        
        // FIXED: Declare bindings inline so they live for the entire creation block
        let layout_bindings = [ubo_binding, sampler_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);
        
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }.unwrap();

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1),
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }.unwrap();

        let alloc_layouts = [descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(&alloc_layouts);
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }.unwrap()[0];

        let mut system = Self { pipeline_layout: vk::PipelineLayout::null(), pipeline: vk::Pipeline::null(), descriptor_set_layout, descriptor_pool, descriptor_set, ubo_buffer };
        
        system.update_descriptor(context, offscreen);

        let pipeline_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&pipeline_layouts);
        system.pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

        let vert_code = Self::read_shader("src/vulkan_logic/compiled_shaders/postprocess.vert.spv")?;
        let frag_code = Self::read_shader("src/vulkan_logic/compiled_shaders/postprocess.frag.spv")?;
        let vert_mod = Self::create_module(&context.device, &vert_code)?;
        let frag_mod = Self::create_module(&context.device, &frag_code)?;
        let entry = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default(); // Fullscreen procedural!
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).cull_mode(vk::CullModeFlags::NONE).line_width(1.0);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false).depth_write_enable(false);
        let color_blend = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A);
        
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_mod).name(entry), 
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_mod).name(entry)
        ];
        
        // FIXED: Extract long-living arrays for the builder functions to consume safely
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachments = [color_blend];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages).vertex_input_state(&vertex_input).input_assembly_state(&assembly).viewport_state(&viewport).rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil).color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info)
            .layout(system.pipeline_layout).render_pass(swapchain_render_pass).subpass(0);

        let pipeline_infos = [pipeline_info];
        system.pipeline = unsafe { context.device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None) }.map_err(|e| anyhow!("{:?}", e.1))?[0];

        unsafe { context.device.destroy_shader_module(vert_mod, None); context.device.destroy_shader_module(frag_mod, None); }
        Ok(system)
    }

    pub fn update_descriptor(&mut self, context: &VulkanContext, offscreen: &OffscreenPass) {
        let buffer_info = vk::DescriptorBufferInfo::default().buffer(self.ubo_buffer.buffer).offset(0).range(std::mem::size_of::<PostProcessUBO>() as u64);
        let image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).image_view(offscreen.color_view).sampler(offscreen.sampler);
        
        // FIXED: Ensure arrays don't get silently dropped by making them local variables first
        let buffer_infos = [buffer_info];
        let image_infos = [image_info];
        
        let writes = [
            vk::WriteDescriptorSet::default().dst_set(self.descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(&buffer_infos),
            vk::WriteDescriptorSet::default().dst_set(self.descriptor_set).dst_binding(1).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(&image_infos),
        ];
        
        unsafe { context.device.update_descriptor_sets(&writes, &[]); }
    }

    pub fn draw(&mut self, context: &VulkanContext, cmd: vk::CommandBuffer, config: &PostProcessConfig, time: f32) -> Result<()> {
        let allocator = context.allocator.as_ref().unwrap();
        
        let ubo = PostProcessUBO {
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
        
        let ubos = [ubo];
        self.ubo_buffer.upload_data(allocator, &ubos)?;

        unsafe {
            context.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            let descriptor_sets = [self.descriptor_set];
            context.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &descriptor_sets, &[]);
            context.device.cmd_draw(cmd, 3, 1, 0, 0); // Fullscreen Quad
        }
        Ok(())
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            if let Some(a) = context.allocator.as_ref() { self.ubo_buffer.destroy(a); }
            context.device.destroy_descriptor_pool(self.descriptor_pool, None);
            context.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            context.device.destroy_pipeline(self.pipeline, None);
            context.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
    
    fn read_shader(p: &str) -> Result<Vec<u8>> { let mut f = File::open(p).map_err(|e| anyhow!("{}", e))?; let mut b = Vec::new(); f.read_to_end(&mut b).unwrap(); Ok(b) }
    fn create_module(d: &ash::Device, c: &[u8]) -> Result<vk::ShaderModule> { let (_, code, _) = unsafe { c.align_to::<u32>() }; unsafe { d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None) }.map_err(|e| anyhow!("{}", e)) }
}