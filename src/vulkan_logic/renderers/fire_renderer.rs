// src/vulkan_logic/renderers/fire_renderer.rs

use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::memory::gpu_buffers::DynamicBuffer;
use crate::fire::ubo::{FireUBO, FireParticleData};
use crate::fire::emitter::FireEmitter;

pub struct FireSystem {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub ubo_buffer: DynamicBuffer,
}

impl FireSystem {
    pub fn new(context: &VulkanContext, render_pass: vk::RenderPass) -> Result<Self> {
        let allocator = match context.allocator.as_ref() { Some(a) => a, None => return Err(anyhow!("No allocator")) };

        let ubo_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<FireUBO>(), vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let bindings = [vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let pool_sizes = [vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }.map_err(|e| anyhow!("{}", e))?[0];

        let buffer_info = vk::DescriptorBufferInfo::default().buffer(ubo_buffer.buffer).offset(0).range(std::mem::size_of::<FireUBO>() as u64);
        let write_set = vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(std::slice::from_ref(&buffer_info));
        unsafe { context.device.update_descriptor_sets(std::slice::from_ref(&write_set), &[]) };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let vert_code = Self::read_shader("src/vulkan_logic/compiled_shaders/fire.vert.spv")?;
        let frag_code = Self::read_shader("src/vulkan_logic/compiled_shaders/fire.frag.spv")?;
        let vert_mod = Self::create_module(&context.device, &vert_code)?;
        let frag_mod = Self::create_module(&context.device, &frag_code)?;
        let entry = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).cull_mode(vk::CullModeFlags::NONE).line_width(1.0);
        
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(true).depth_write_enable(false).depth_compare_op(vk::CompareOp::LESS);
        
        // CRITICAL AAA DIFFERENCE: Fire uses ADDITIVE blending, not Alpha blending!
        let blend_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE) // Additive magic!
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_mod).name(entry), 
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_mod).name(entry)
        ];
        
        let blend_attachments = [blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input).input_assembly_state(&assembly).viewport_state(&viewport).rasterization_state(&rasterizer)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil).color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout).render_pass(render_pass).subpass(0);

        let pipeline = unsafe { context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None) }.map_err(|e| anyhow!("{:?}", e.1))?[0];

        unsafe { context.device.destroy_shader_module(vert_mod, None); context.device.destroy_shader_module(frag_mod, None); }

        Ok(Self { pipeline_layout, pipeline, descriptor_set_layout, descriptor_pool, descriptor_set, ubo_buffer })
    }

    pub fn draw(&mut self, context: &VulkanContext, cmd: vk::CommandBuffer, emitter: &FireEmitter, view_proj: glam::Mat4, camera_view: glam::Mat4) -> Result<()> {
        let allocator = match context.allocator.as_ref() { Some(a) => a, None => return Err(anyhow!("No allocator")) };
        if emitter.particles.is_empty() { return Ok(()); }

        let inv_view = camera_view.inverse();

        let mut ubo = FireUBO {
            view_proj,
            camera_right: inv_view.x_axis,
            camera_up: inv_view.y_axis,
            particle_count: emitter.particles.len() as u32,
            _pad: [0; 3],
            particles: [FireParticleData::default(); 500],
        };

        for (i, p) in emitter.particles.iter().enumerate().take(500) {
            ubo.particles[i] = FireParticleData {
                position: glam::Vec4::new(p.position.x, p.position.y, p.position.z, p.scale),
                color: glam::Vec4::new(p.color[0], p.color[1], p.color[2], p.alpha),
            };
        }

        self.ubo_buffer.upload_data(allocator, &[ubo])?;

        unsafe {
            context.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            context.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, std::slice::from_ref(&self.descriptor_set), &[]);
            context.device.cmd_draw(cmd, 6, ubo.particle_count, 0, 0);
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
    
    fn read_shader(p: &str) -> Result<Vec<u8>> { let mut f = File::open(p).map_err(|e| anyhow!("{}", e))?; let mut b = Vec::new(); f.read_to_end(&mut b).map_err(|e| anyhow!("{}", e))?; Ok(b) }
    fn create_module(d: &ash::Device, c: &[u8]) -> Result<vk::ShaderModule> { let (p, code, s) = unsafe { c.align_to::<u32>() }; if !p.is_empty() || !s.is_empty() { return Err(anyhow!("Align err")); } unsafe { d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None) }.map_err(|e| anyhow!("{}", e)) }
}