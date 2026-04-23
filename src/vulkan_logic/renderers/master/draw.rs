// src/vulkan_logic/renderers/master/draw.rs

use anyhow::Result;
use winit::window::Window;
use super::renderer::MasterRenderer;

use crate::assets::model::Model; 

use crate::lights::global::directional::GlobalLight;
use crate::lights::spawnable::spot::SpotLight; 
use crate::lights::spawnable::point::PointLight;

use crate::skybox::config::SkyboxConfig;
use crate::clouds::config::CloudConfig;
use crate::water::config::RiverConfig;
use crate::smoke::emitter::SmokeEmitter;
use crate::fire::emitter::FireEmitter;
use crate::weather::emitter::WeatherEmitter;
use crate::volumetrics::fog_config::FogConfig;

impl MasterRenderer {
    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame(
        &mut self, 
        window: &Window, 
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
        pixels_per_point: f32,
        is_playing: bool,
        camera_pos: glam::Vec3,
        camera_view_matrix: glam::Mat4,           
        ambient_color: [f32; 3],
        ambient_intensity: f32,
        global_lights: &[GlobalLight],
        spot_lights: &[SpotLight],         
        point_lights: &[PointLight],       
        visible_objects: &[Model],  
        skybox_config: &SkyboxConfig,
        cloud_config: &CloudConfig, 
        river_configs: &[RiverConfig], // Updated Array routing
        fog_config: &FogConfig,
        smoke_emitters: &[SmokeEmitter], // Updated Array routing
        fire_emitters: &[FireEmitter], // Updated Array routing
        weather_emitter: &WeatherEmitter,
        time: f32,                  
    ) -> Result<()> {
        
        self.wait_fences()?;

        let frame_data = self.prepare_data(
            is_playing,
            camera_pos,
            ambient_color,
            ambient_intensity,
            global_lights,
            spot_lights,
            point_lights,
            visible_objects,
            fog_config,
        )?;

        self.update_egui_textures(textures_delta)?;

        let img_idx_opt = self.acquire_image(window)?;
        let img_idx = match img_idx_opt {
            Some(idx) => idx,
            None => return Ok(()),
        };

        self.begin_command_buffer()?;

        if is_playing && !frame_data.draw_calls.is_empty() {
            self.record_shadow_pass(&frame_data)?;
        }

        self.record_main_pass(
            img_idx,
            is_playing,
            camera_pos,
            camera_view_matrix,
            &frame_data,
            skybox_config,
            cloud_config,
            river_configs,
            smoke_emitters,
            fire_emitters,
            weather_emitter,
            clipped_primitives,
            pixels_per_point,
            time,
        )?;

        self.end_command_buffer()?;
        self.submit_and_present(img_idx, window)?;

        Ok(())
    }
}