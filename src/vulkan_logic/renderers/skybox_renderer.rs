// src/vulkan_logic/renderers/skybox_renderer.rs
use anyhow::{anyhow, Result};
use ash::vk;
use std::fs::File;
use std::io::Read;
use std::ffi::CStr;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::memory::gpu_buffers::DynamicBuffer;
use crate::vulkan_logic::config::paths; // FIXED IMPORT

use crate::skybox::ubo::{SkyboxUBO, SkyboxDiscData, SkyboxCrescentData, SkyboxPushConstants};
use crate::skybox::config::SkyboxConfig;

pub struct SkyboxSystem {
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub ubo_buffer: DynamicBuffer,
}

impl SkyboxSystem {
    pub fn new(context: &VulkanContext, render_pass: vk::RenderPass) -> Result<Self> {
        let allocator = match context.allocator.as_ref() {
            Some(alloc) => alloc,
            None => return Err(anyhow!("Vulkan memory allocator was not initialized")),
        };

        let ubo_binding = vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let bindings = [ubo_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe { context.device.create_descriptor_set_layout(&layout_info, None) }.map_err(|e| anyhow!("Failed to create skybox descriptor set layout: {}", e))?;

        let pool_sizes = [vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1)];
        let pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1);
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None) }.map_err(|e| anyhow!("Failed to create skybox descriptor pool: {}", e))?;

        let alloc_info = vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&alloc_info) }.map_err(|e| anyhow!("Failed to allocate skybox descriptor sets: {}", e))?[0];

        let ubo_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<SkyboxUBO>(), vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let buffer_info = vk::DescriptorBufferInfo::default().buffer(ubo_buffer.buffer).offset(0).range(std::mem::size_of::<SkyboxUBO>() as u64);
        let write_set = vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(std::slice::from_ref(&buffer_info));
        unsafe { context.device.update_descriptor_sets(std::slice::from_ref(&write_set), &[]) };

        let push_constant_range = vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::VERTEX).offset(0).size(std::mem::size_of::<SkyboxPushConstants>() as u32);
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&descriptor_set_layout)).push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout = unsafe { context.device.create_pipeline_layout(&pipeline_layout_info, None) }.map_err(|e| anyhow!("Failed to create skybox pipeline layout: {}", e))?;

        // USES CENTRALIZED SHADER PATHS
        let vert_shader_code = Self::read_shader_file(paths::SKYBOX_VERT)?;
        let frag_shader_code = Self::read_shader_file(paths::SKYBOX_FRAG)?;

        let vert_module = Self::create_shader_module(&context.device, &vert_shader_code)?;
        let frag_module = Self::create_shader_module(&context.device, &frag_shader_code)?;
        let main_func = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vert_module).name(main_func),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(frag_module).name(main_func),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).cull_mode(vk::CullModeFlags::NONE).line_width(1.0);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false).depth_write_enable(false);
            
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A);
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default().attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages).vertex_input_state(&vertex_input_info).input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state).rasterization_state(&rasterizer).multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil).color_blend_state(&color_blending).dynamic_state(&dynamic_state)
            .layout(pipeline_layout).render_pass(render_pass).subpass(0);

        let pipeline = unsafe { context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None) }.map_err(|e| anyhow!("Failed to create skybox pipeline: {:?}", e.1))?[0];

        unsafe {
            context.device.destroy_shader_module(vert_module, None);
            context.device.destroy_shader_module(frag_module, None);
        }

        Ok(Self { pipeline_layout, pipeline, descriptor_set_layout, descriptor_pool, descriptor_set, ubo_buffer })
    }

    pub fn draw(&mut self, context: &VulkanContext, command_buffer: vk::CommandBuffer, extent: &vk::Extent2D, config: &SkyboxConfig, camera_view_matrix: glam::Mat4) -> Result<()> {
        let allocator = match context.allocator.as_ref() {
            Some(alloc) => alloc,
            None => return Err(anyhow!("Memory allocator missing during skybox draw")),
        };

        let mut ubo = SkyboxUBO {
            zenith_color: glam::Vec4::new(config.zenith_color[0], config.zenith_color[1], config.zenith_color[2], 1.0),
            horizon_color: glam::Vec4::new(config.horizon_color[0], config.horizon_color[1], config.horizon_color[2], 1.0),
            ground_color: glam::Vec4::new(config.ground_color[0], config.ground_color[1], config.ground_color[2], 1.0),
            disc_count: 0, crescent_count: 0, _pad: [0; 2],
            discs: [SkyboxDiscData::default(); 5], crescents: [SkyboxCrescentData::default(); 5],
        };

        for disc in &config.discs {
            if ubo.disc_count < 5 {
                ubo.discs[ubo.disc_count as usize] = SkyboxDiscData {
                    direction: glam::Vec4::new(disc.direction.x, disc.direction.y, disc.direction.z, disc.angular_size),
                    color: glam::Vec4::new(disc.color[0], disc.color[1], disc.color[2], disc.glow_intensity),
                };
                ubo.disc_count += 1;
            }
        }

        for crescent in &config.crescents {
            if ubo.crescent_count < 5 {
                ubo.crescents[ubo.crescent_count as usize] = SkyboxCrescentData {
                    direction: glam::Vec4::new(crescent.direction.x, crescent.direction.y, crescent.direction.z, crescent.angular_size),
                    color: glam::Vec4::new(crescent.color[0], crescent.color[1], crescent.color[2], crescent.cutout_size),
                    cutout_offset: glam::Vec4::new(crescent.cutout_offset.x, crescent.cutout_offset.y, crescent.cutout_offset.z, 0.0),
                };
                ubo.crescent_count += 1;
            }
        }

        self.ubo_buffer.upload_data(allocator, &[ubo])?;

        let mut sky_view = camera_view_matrix;
        sky_view.w_axis = glam::Vec4::new(0.0, 0.0, 0.0, 1.0); 
        
        let aspect = extent.width as f32 / extent.height as f32;
        let mut proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 1000.0);
        proj.y_axis.y *= -1.0; 
        
        let inv_view_proj = (proj * sky_view).inverse();
        let pc = SkyboxPushConstants { inv_view_proj };

        unsafe {
            context.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            context.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, std::slice::from_ref(&self.descriptor_set), &[]);
            let pc_bytes = std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<SkyboxPushConstants>());
            context.device.cmd_push_constants(command_buffer, self.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, pc_bytes);
            context.device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }

        Ok(())
    }

    pub fn destroy(&mut self, context: &VulkanContext) {
        unsafe {
            if let Some(alloc) = context.allocator.as_ref() { self.ubo_buffer.destroy(alloc); }
            context.device.destroy_descriptor_pool(self.descriptor_pool, None);
            context.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            context.device.destroy_pipeline(self.pipeline, None);
            context.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }

    fn read_shader_file(path: &str) -> Result<Vec<u8>> {
        let mut file = File::open(path).map_err(|e| anyhow!("Failed to open skybox shader at {}: {}", path, e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| anyhow!("Failed to read skybox shader: {}", e))?;
        Ok(buffer)
    }

    fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let (prefix, code_u32, suffix) = unsafe { code.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() { return Err(anyhow!("Shader bytecode misaligned")); }
        let create_info = vk::ShaderModuleCreateInfo::default().code(code_u32);
        unsafe { device.create_shader_module(&create_info, None).map_err(|e| anyhow!("Failed to create shader module: {}", e)) }
    }
}