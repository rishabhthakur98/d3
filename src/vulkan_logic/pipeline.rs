// src/vulkan_logic/pipeline.rs

use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use super::context::VulkanContext;
use crate::vulkan_logic::vertex_setup::VertexSetup;
use crate::backface_cull_config::{DEFAULT_CULL_MODE, CULL_MODE_NONE, CULL_MODE_CW};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PushConstants {
    pub view_proj: glam::Mat4, 
    pub model: glam::Mat4,     
}

pub struct VulkanPipeline {
    pub layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout, // NEW
}

impl VulkanPipeline {
    pub fn new(context: &VulkanContext, render_pass: vk::RenderPass) -> Result<Self> {
        let vert_shader_code = Self::read_shader_file("src/vulkan_logic/compiled_shaders/triangle.vert.spv")?;
        let frag_shader_code = Self::read_shader_file("src/vulkan_logic/compiled_shaders/triangle.frag.spv")?;

        let vert_shader_module = Self::create_shader_module(&context.device, &vert_shader_code)?;
        let frag_shader_module = Self::create_shader_module(&context.device, &frag_shader_code)?;

        let main_function_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(main_function_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(main_function_name),
        ];

        let binding_descriptions = [VertexSetup::get_binding_description()];
        let attribute_descriptions = VertexSetup::get_attribute_descriptions();
        
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&dynamic_states);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let (cull_mode, front_face) = match DEFAULT_CULL_MODE {
            CULL_MODE_NONE => (vk::CullModeFlags::NONE, vk::FrontFace::COUNTER_CLOCKWISE),
            CULL_MODE_CW => (vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE),
            _ => (vk::CullModeFlags::BACK, vk::FrontFace::COUNTER_CLOCKWISE), 
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(cull_mode)
            .front_face(front_face)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)               
            .depth_write_enable(true)              
            .depth_compare_op(vk::CompareOp::LESS) 
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | 
                vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // --- NEW: Descriptor Set Layout for the Uniform Buffer ---
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let ubo_layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&ubo_binding));

        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&ubo_layout_info, None) }
            .map_err(|e| anyhow!("Failed to create descriptor set layout: {}", e))?;

        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<PushConstants>() as u32); 

        // Apply the Descriptor Layout to the Pipeline Layout
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(|e| anyhow!("Failed to create pipeline layout: {}", e))?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil_state) 
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let graphics_pipeline = unsafe {
            context.device.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None)
        }.map_err(|e| anyhow!("Failed to create graphics pipeline: {:?}", e.1))?[0];

        unsafe {
            context.device.destroy_shader_module(vert_shader_module, None);
            context.device.destroy_shader_module(frag_shader_module, None);
        }

        Ok(Self { layout: pipeline_layout, graphics_pipeline, descriptor_set_layout })
    }

    fn read_shader_file(path: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path).map_err(|e| anyhow!("Failed to open shader file at {}: {}", path, e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| anyhow!("Failed to read shader file: {}", e))?;
        Ok(buffer)
    }

    fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let (prefix, code_u32, suffix) = unsafe { code.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() { 
            return Err(anyhow!("Shader bytecode misaligned")); 
        }
        let create_info = vk::ShaderModuleCreateInfo::default().code(code_u32);
        unsafe { device.create_shader_module(&create_info, None) }
            .map_err(|e| anyhow!("Failed to create shader module: {}", e))
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }
}