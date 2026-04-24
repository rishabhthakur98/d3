// src/vulkan_logic/pipelines/main_pipeline.rs

use anyhow::{anyhow, Result};
use ash::vk;
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::config::cull::BackfaceCullConfig;
use crate::vulkan_logic::config::paths;
use crate::vulkan_logic::memory::vertex_setup::VertexSetup;

// Expose PushConstants so the master renderer pass logic can bind variables to shaders
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PushConstants {
    pub view_proj: glam::Mat4,
    pub model: glam::Mat4,
    pub light_space_matrix: glam::Mat4,
}

pub struct MainPipeline {
    pub layout: vk::PipelineLayout,
    pub graphics_pipeline: vk::Pipeline,
    pub shadow_pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
}

impl MainPipeline {
    pub fn new(context: &VulkanContext, render_pass: vk::RenderPass, shadow_render_pass: vk::RenderPass) -> Result<Self> {
        // --- 1. DESCRIPTORS & LAYOUTS ---
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let shadow_map_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = [ubo_binding, shadow_map_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }
            .map_err(|e| anyhow!("Failed to create descriptor set layout: {}", e))?;

        let push_constant = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<PushConstants>() as u32);

        let set_layouts = [descriptor_set_layout];
        let push_constants = [push_constant];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constants);

        let layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(|e| anyhow!("Failed to create pipeline layout: {}", e))?;


        // --- 2. PIPELINE PREPARATION LOGIC ---
        let binding_descriptions = [VertexSetup::get_binding_description()];
        let attribute_descriptions = VertexSetup::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let entry_point_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };


        // --- 3. MAIN GRAPHICS PIPELINE ---
        let vert_code = Self::read_shader(paths::MAIN_VERT)?;
        let frag_code = Self::read_shader(paths::MAIN_FRAG)?;

        let vert_module = Self::create_shader_module(&context.device, &vert_code)?;
        let frag_module = Self::create_shader_module(&context.device, &frag_code)?;

        let main_shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(entry_point_name),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(entry_point_name),
        ];

        let cull_config = BackfaceCullConfig::default();
        let main_rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(cull_config.get_cull_mode())
            .front_face(cull_config.get_front_face())
            .depth_bias_enable(false);

        let main_depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));

        let main_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&main_shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&main_rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&main_depth_stencil)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);


        // --- 4. SHADOW PASS PIPELINE ---
        let shadow_vert_code = Self::read_shader(paths::SHADOW_VERT)?;
        let shadow_vert_module = Self::create_shader_module(&context.device, &shadow_vert_code)?;

        let shadow_shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(shadow_vert_module).name(entry_point_name),
        ];

        let shadow_rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(true)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::FRONT) // Extremely important: prevents 'Peter Panning' shadow separation
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(true)
            .depth_bias_constant_factor(1.25)
            .depth_bias_slope_factor(1.75);

        let shadow_depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

        let shadow_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shadow_shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&shadow_rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&shadow_depth_stencil)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(shadow_render_pass)
            .subpass(0);

        // --- 5. BATCH COMPILE PIPELINES ---
        let pipelines = unsafe {
            context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[main_pipeline_info, shadow_pipeline_info], None)
        }.map_err(|e| anyhow!("Failed to create graphics pipelines: {:?}", e.1))?;

        unsafe {
            context.device.destroy_shader_module(vert_module, None);
            context.device.destroy_shader_module(frag_module, None);
            context.device.destroy_shader_module(shadow_vert_module, None);
        }

        Ok(Self {
            layout,
            graphics_pipeline: pipelines[0],
            shadow_pipeline: pipelines[1],
            descriptor_set_layout,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline(self.shadow_pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }

    fn read_shader(path: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path).map_err(|e| anyhow!("Failed to open shader {}: {}", path, e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| anyhow!("Failed to read shader {}: {}", path, e))?;
        Ok(buffer)
    }

    fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let (prefix, aligned_code, suffix) = unsafe { code.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            return Err(anyhow!("Shader code is not perfectly aligned to u32 boundaries"));
        }

        let create_info = vk::ShaderModuleCreateInfo::default().code(aligned_code);
        unsafe { device.create_shader_module(&create_info, None) }
            .map_err(|e| anyhow!("Failed to create shader module: {}", e))
    }
}