// src/app/engine_state.rs

use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

use crate::game::menu::MenuSystem;
use crate::game::world01::controls::InputState;
use crate::game::world01::world_streamer::WorldStreamer;
use crate::game::world01::camera::FreeformCamera;
use crate::lights::global::directional::GlobalLight;
use crate::skybox::config::SkyboxConfig;
use crate::clouds::config::CloudConfig;
use crate::volumetrics::fog_config::FogConfig; 
use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::renderers::master::MasterRenderer;

pub struct EngineApp {
    pub renderer: Option<MasterRenderer>,
    pub context: Option<Arc<VulkanContext>>,
    pub window: Option<Arc<Window>>,
    
    pub menu: MenuSystem,
    pub egui_ctx: egui::Context,
    pub egui_state: Option<egui_winit::State>,
    
    pub is_playing: bool,
    pub input_state: InputState,
    pub world_streamer: WorldStreamer,
    pub camera: FreeformCamera,
    
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
    pub global_lights: Vec<GlobalLight>,
    
    pub skybox_config: SkyboxConfig,
    pub cloud_config: CloudConfig,
    pub fog_config: FogConfig, 
    
    pub engine_start_time: Instant, 
    pub last_update_time: Instant, 
    pub last_frame_time: Instant,  
}

impl Default for EngineApp {
    fn default() -> Self {
        Self {
            renderer: None, context: None, window: None, 
            menu: MenuSystem::new(), egui_ctx: egui::Context::default(), egui_state: None,
            is_playing: false, input_state: InputState::default(),
            world_streamer: WorldStreamer::new(), camera: FreeformCamera::default(),
            ambient_color: [0.0, 0.0, 0.0], ambient_intensity: 0.0, global_lights: Vec::new(),
            skybox_config: SkyboxConfig::default(), 
            cloud_config: CloudConfig::default(),
            fog_config: FogConfig::default(), 
            engine_start_time: Instant::now(), last_update_time: Instant::now(), last_frame_time: Instant::now(),
        }
    }
}