// src/vulkan_logic/renderers/master/renderer.rs
use anyhow::{anyhow, Result};
use ash::vk;
use egui_ash_renderer::Renderer as EguiRenderer;
use egui_ash_renderer::Options as EguiOptions;
use winit::window::Window;
use std::sync::Arc;

use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::core::swapchain::SwapchainManager; 
use crate::vulkan_logic::core::sync_objects::SyncObjects;
use crate::vulkan_logic::pipelines::main_pipeline::MainPipeline;
use crate::vulkan_logic::memory::gpu_buffers::DynamicBuffer;
use crate::vulkan_logic::passes::shadow_pass::ShadowPass;
use crate::lights::core::ubo::LightUBO;

use crate::vulkan_logic::renderers::skybox_renderer::SkyboxSystem;
use crate::vulkan_logic::renderers::river_renderer::RiverSystem;
use crate::vulkan_logic::renderers::smoke_renderer::SmokeSystem;
use crate::vulkan_logic::renderers::cloud_renderer::CloudSystem;
use crate::vulkan_logic::renderers::fire_renderer::FireSystem;
use crate::vulkan_logic::renderers::weather_renderer::WeatherSystem;

use crate::postprocessing::offscreen::OffscreenPass;
use crate::postprocessing::system::PostProcessSystem;

#[derive(Clone, Debug)]
pub(crate) struct DrawCall {
    pub index_start: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
    pub transform: glam::Mat4,
}

pub struct MasterRenderer {
    pub is_resized: bool,
    pub egui_renderer: EguiRenderer,
    pub(crate) context: Arc<VulkanContext>,
    pub(crate) swapchain_mgr: SwapchainManager,
    pub(crate) sync: SyncObjects,
    
    pub(crate) offscreen_pass: OffscreenPass, 
    pub(crate) post_process_system: PostProcessSystem,

    pub(crate) shadow_pass: ShadowPass, 
    pub(crate) pipeline: MainPipeline,
    
    pub(crate) skybox_system: SkyboxSystem, 
    pub(crate) cloud_system: CloudSystem, 
    pub(crate) river_system: RiverSystem, 
    pub(crate) smoke_system: SmokeSystem, 
    pub(crate) fire_system: FireSystem,
    pub(crate) weather_system: WeatherSystem,
    
    pub(crate) vertex_buffer: DynamicBuffer,
    pub(crate) index_buffer: DynamicBuffer,
    pub(crate) uniform_buffer: DynamicBuffer,
    pub(crate) descriptor_pool: vk::DescriptorPool,
    pub(crate) descriptor_set: vk::DescriptorSet,
}

impl MasterRenderer {
    pub fn new(context: Arc<VulkanContext>, window: &Window) -> Result<Self> {
        let swapchain_mgr = SwapchainManager::new(&context, window)?;
        let sync = SyncObjects::new(&context)?;

        let egui_renderer = EguiRenderer::with_default_allocator(
            &context.instance, context.physical_device, context.device.clone(), swapchain_mgr.render_pass,
            EguiOptions { srgb_framebuffer: false, ..Default::default() },
        ).map_err(|e| anyhow!("Failed to init egui: {}", e))?;

        let shadow_pass = ShadowPass::new(&context)?;
        let offscreen_pass = OffscreenPass::new(&context, swapchain_mgr.extent)?;

        let pipeline = MainPipeline::new(&context, offscreen_pass.render_pass, shadow_pass.render_pass)?;
        let skybox_system = SkyboxSystem::new(&context, offscreen_pass.render_pass)?;
        let cloud_system = CloudSystem::new(&context, offscreen_pass.render_pass)?; 
        let river_system = RiverSystem::new(&context, offscreen_pass.render_pass)?;
        let smoke_system = SmokeSystem::new(&context, offscreen_pass.render_pass)?; 
        let fire_system = FireSystem::new(&context, offscreen_pass.render_pass)?; 
        let weather_system = WeatherSystem::new(&context, offscreen_pass.render_pass)?;
        
        let post_process_system = PostProcessSystem::new(&context, swapchain_mgr.render_pass, &offscreen_pass)?;
        
        let allocator = context.allocator.as_ref().unwrap();

        let vertex_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<crate::assets::model::Vertex>() * 10000, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let index_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<u32>() * 10000, vk::BufferUsageFlags::INDEX_BUFFER)?;

        let pool_sizes = [
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1),
            vk::DescriptorPoolSize::default().ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).descriptor_count(1),
        ];
        let descriptor_pool = unsafe { context.device.create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(1), None) }.unwrap();
        let descriptor_set = unsafe { context.device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default().descriptor_pool(descriptor_pool).set_layouts(std::slice::from_ref(&pipeline.descriptor_set_layout))) }.unwrap()[0];
        let uniform_buffer = DynamicBuffer::new(allocator, std::mem::size_of::<LightUBO>(), vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let uniform_buffer_info = vk::DescriptorBufferInfo::default().buffer(uniform_buffer.buffer).offset(0).range(std::mem::size_of::<LightUBO>() as u64);
        let shadow_image_info = vk::DescriptorImageInfo::default().image_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL).image_view(shadow_pass.depth_view).sampler(shadow_pass.sampler);
        let write_sets = [
            vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(0).dst_array_element(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(std::slice::from_ref(&uniform_buffer_info)),
            vk::WriteDescriptorSet::default().dst_set(descriptor_set).dst_binding(1).dst_array_element(0).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(&shadow_image_info)),
        ];
        unsafe { context.device.update_descriptor_sets(&write_sets, &[]) };

        Ok(Self { 
            is_resized: false, egui_renderer, context, swapchain_mgr, sync, offscreen_pass, post_process_system, shadow_pass, pipeline, 
            skybox_system, cloud_system, river_system, smoke_system, fire_system, weather_system, vertex_buffer, index_buffer, uniform_buffer, descriptor_pool, descriptor_set
        })
    }
}

impl Drop for MasterRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.context.device.device_wait_idle();
            if let Some(allocator) = self.context.allocator.as_ref() {
                self.vertex_buffer.destroy(allocator);
                self.index_buffer.destroy(allocator);
                self.uniform_buffer.destroy(allocator); 
            }
            
            self.post_process_system.destroy(&self.context);
            self.offscreen_pass.destroy(&self.context);
            
            self.skybox_system.destroy(&self.context);
            self.cloud_system.destroy(&self.context); 
            self.river_system.destroy(&self.context); 
            self.smoke_system.destroy(&self.context); 
            self.fire_system.destroy(&self.context);
            self.weather_system.destroy(&self.context);
            
            self.context.device.destroy_descriptor_pool(self.descriptor_pool, None); 
            self.shadow_pass.destroy(&self.context); 
            self.pipeline.destroy(&self.context.device);
            self.sync.destroy(&self.context.device);
            self.swapchain_mgr.destroy(&self.context);
        }
    }
}