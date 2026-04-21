// src/vulkan_logic/pipelines/main_pipeline.rs
use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::memory::vertex_setup::VertexSetup;
use crate::backface_cull_config::{DEFAULT_CULL_MODE, CULL_MODE_NONE, CULL_MODE_CW};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PushConstants {
    pub view_proj: glam::Mat4, 
    pub model: glam::Mat4,     
    pub light_space_matrix: glam::Mat4, 
}

pub struct MainPipeline {
    pub layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub shadow_pipeline: vk::Pipeline, // ADDED BACK
    pub descriptor_set_layout: vk::DescriptorSetLayout, 
}

impl MainPipeline {
    pub fn new(context: &VulkanContext, main_render_pass: vk::RenderPass, shadow_render_pass: vk::RenderPass) -> Result<Self> {
        let vert_code = Self::read_shader("src/vulkan_logic/compiled_shaders/triangle.vert.spv")?;
        let frag_code = Self::read_shader("src/vulkan_logic/compiled_shaders/triangle.frag.spv")?;
        let shadow_vert_code = Self::read_shader("src/vulkan_logic/compiled_shaders/shadow.vert.spv")?;

        let vert_mod = Self::create_module(&context.device, &vert_code)?;
        let frag_mod = Self::create_module(&context.device, &frag_code)?;
        let shadow_vert_mod = Self::create_module(&context.device, &shadow_vert_code)?;
        
        let entry = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let ubo_binding = vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);
        let sampler_binding = vk::DescriptorSetLayoutBinding::default().binding(1).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let layout_bindings = [ubo_binding, sampler_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let pc_range = vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::VERTEX).offset(0).size(std::mem::size_of::<PushConstants>() as u32); 
        let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&descriptor_set_layout)).push_constant_ranges(std::slice::from_ref(&pc_range));
        let layout = unsafe { context.device.create_pipeline_layout(&layout_info, None) }.map_err(|e| anyhow!("{}", e))?;

        let bindings = [VertexSetup::get_binding_description()];
        let attributes = VertexSetup::get_attribute_descriptions();
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default().vertex_binding_descriptions(&bindings).vertex_attribute_descriptions(&attributes);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        
        let (cull_mode, front_face) = match DEFAULT_CULL_MODE {
            CULL_MODE_NONE => (vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE),
            CULL_MODE_CW => (vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE),
            _ => (vk::CullModeFlags::BACK, vk::FrontFace::COUNTER_CLOCKWISE), 
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).line_width(1.0).cull_mode(cull_mode).front_face(front_face);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(true).depth_write_enable(true).depth_compare_op(vk::CompareOp::LESS);
        let color_blend = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A);
        
        let blend_attachments = [color_blend];
        let blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_mod).name(entry),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_mod).name(entry),
        ];

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let main_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages).vertex_input_state(&vertex_input).input_assembly_state(&input_assembly).viewport_state(&viewport_state)
            .rasterization_state(&rasterizer).multisample_state(&multisample_info)
            .depth_stencil_state(&depth_stencil).color_blend_state(&blending).dynamic_state(&dynamic_state_info).layout(layout).render_pass(main_render_pass).subpass(0);

        // --- Shadow Pipeline ---
        let shadow_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(shadow_vert_mod).name(entry),
        ];

        let shadow_rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false).rasterizer_discard_enable(false).polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0).cull_mode(vk::CullModeFlags::NONE).front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(true).depth_bias_constant_factor(1.25).depth_bias_slope_factor(1.75);

        let shadow_depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true).depth_write_enable(true).depth_compare_op(vk::CompareOp::LESS);

        let shadow_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(&[]);

        let shadow_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shadow_stages).vertex_input_state(&vertex_input).input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state).rasterization_state(&shadow_rasterizer).multisample_state(&multisample_info)
            .depth_stencil_state(&shadow_depth_stencil).color_blend_state(&shadow_blending).dynamic_state(&dynamic_state_info).layout(layout).render_pass(shadow_render_pass).subpass(0);

        let pipelines = unsafe { context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[main_pipeline_info, shadow_pipeline_info], None) }.map_err(|e| anyhow!("{:?}", e.1))?;

        unsafe { 
            context.device.destroy_shader_module(vert_mod, None); 
            context.device.destroy_shader_module(frag_mod, None); 
            context.device.destroy_shader_module(shadow_vert_mod, None);
        }
        Ok(Self { layout, graphics_pipeline: pipelines[0], shadow_pipeline: pipelines[1], descriptor_set_layout })
    }

    fn read_shader(path: &str) -> Result<Vec<u8>> {
        let mut f = File::open(path).map_err(|e| anyhow!("{}", e))?;
        let mut buf = Vec::new(); f.read_to_end(&mut buf).map_err(|e| anyhow!("{}", e))?; Ok(buf)
    }
    fn create_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let (p, c, s) = unsafe { code.align_to::<u32>() };
        if !p.is_empty() || !s.is_empty() { return Err(anyhow!("Misaligned")); }
        unsafe { device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(c), None) }.map_err(|e| anyhow!("{}", e))
    }
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe { 
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None); 
            device.destroy_pipeline(self.graphics_pipeline, None); 
            device.destroy_pipeline(self.shadow_pipeline, None);
            device.destroy_pipeline_layout(self.layout, None); 
        }
    }
}