// src/vulkan_logic/renderers/master/draw.rs

use anyhow::Result;
use winit::window::Window;
use super::renderer::MasterRenderer;

use crate::geometrical_shapes::game_object::GameObject;
use crate::light::global::GlobalLight;
use crate::light::spot::SpotLight; 
use crate::light::point::PointLight;

use crate::skybox::config::SkyboxConfig;
use crate::clouds::config::CloudConfig;
use crate::water::config::RiverConfig;
use crate::smoke::emitter::SmokeEmitter;
use crate::fire::emitter::FireEmitter;
use crate::weather::emitter::WeatherEmitter;
use crate::volumetrics::fog_config::FogConfig;

impl MasterRenderer {
    /// High-level orchestration of the frame.
    /// Notice how beautifully short and readable this is now that logic is segregated!
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
        visible_objects: &[GameObject], 
        skybox_config: &SkyboxConfig,
        cloud_config: &CloudConfig, 
        river_config: &RiverConfig, 
        fog_config: &FogConfig,
        smoke_emitter: &SmokeEmitter, 
        fire_emitter: &FireEmitter, 
        weather_emitter: &WeatherEmitter,
        time: f32,                  
    ) -> Result<()> {
        
        // 1. Ensure GPU has finished the previous workload
        self.wait_fences()?;

        // 2. Collate, compute, and upload all dynamic memory buffers (Vertices, Indices, Lighting UBOs)
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

        // 3. Process new UI textures immediately before recording commands
        self.update_egui_textures(textures_delta)?;

        // 4. Secure our canvas to draw on. If we miss (e.g. window resize), exit early cleanly.
        let img_idx_opt = self.acquire_image(window)?;
        let img_idx = match img_idx_opt {
            Some(idx) => idx,
            None => return Ok(()),
        };

        // 5. Open the command buffer for recording
        self.begin_command_buffer()?;

        // 6. Draw the Shadow Map Pass (Depth-Only Rendering)
        if is_playing && !frame_data.draw_calls.is_empty() {
            self.record_shadow_pass(&frame_data)?;
        }

        // 7. Draw the complete Forward Main Pass (Geometry, Lighting, UI)
        self.record_main_pass(
            img_idx,
            is_playing,
            camera_pos,
            camera_view_matrix,
            &frame_data,
            skybox_config,
            cloud_config,
            river_config,
            smoke_emitter,
            fire_emitter,
            weather_emitter,
            clipped_primitives,
            pixels_per_point,
            time,
        )?;

        // 8. Seal the command buffer and dispatch it to the GPU queues
        self.end_command_buffer()?;
        self.submit_and_present(img_idx, window)?;

        Ok(())
    }
}