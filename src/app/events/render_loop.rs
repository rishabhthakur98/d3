// src/app/events/render_loop.rs

use std::time::{Instant, Duration};

use crate::app::engine_state::EngineApp;
use crate::{game, frame_config};

impl EngineApp {
    // --------------------------------------------------------------------------------
    // The heaviest function in the CPU timeline. Calculates physics ticks, gathers UI
    // states, parses visibility, and feeds everything over to the MasterRenderer.
    // --------------------------------------------------------------------------------
    pub(crate) fn process_redraw(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_update_time).as_secs_f32();
        self.last_update_time = now; 
        
        let current_time = self.engine_start_time.elapsed().as_secs_f32(); 
        
        if let (Some(renderer), Some(window), Some(state)) = (&mut self.renderer, &self.window, &mut self.egui_state) {
            
            // 1. Step Physical World and Spatial Culling
            let (visible_objects, active_spots, active_points) = if self.is_playing {
                game::world01::controls::update_camera_position(&mut self.camera, &self.input_state, delta_time);
                
                self.smoke_emitter.tick(delta_time); 
                self.fire_emitter.tick(delta_time); 
                self.weather_emitter.tick(delta_time); 
                
                self.world_streamer.get_visible_objects(self.camera.position)
            } else { 
                (Vec::new(), Vec::new(), Vec::new()) 
            };

            // 2. Map and Tessellate UI
            let raw_input = state.take_egui_input(window.as_ref());
            let full_output = if !self.is_playing {
                let options = self.menu.get_current_options(); 
                let cursor = self.menu.cursor_index;
                
                self.egui_ctx.run(raw_input, |ctx| {
                    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(0, 0, 180))).show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(150.0); 
                            ui.heading(egui::RichText::new("D3 ENGINE").size(80.0).color(egui::Color32::WHITE)); 
                            ui.add_space(80.0);
                            
                            for (i, opt) in options.iter().enumerate() {
                                let text = if i == cursor { format!("-> [ {} ] <-", opt) } else { opt.to_string() };
                                let color = if i == cursor { egui::Color32::YELLOW } else { egui::Color32::GRAY };
                                ui.label(egui::RichText::new(text).size(40.0).color(color)); 
                                ui.add_space(20.0);
                            }
                        });
                    });
                })
            } else { 
                self.egui_ctx.run(raw_input, |_| {}) 
            };

            state.handle_platform_output(window.as_ref(), full_output.platform_output);
            let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

            // 3. Issue dispatch to the heavily decoupled Master Renderer
            if let Err(e) = renderer.draw_frame(
                window.as_ref(), 
                &clipped_primitives, 
                &full_output.textures_delta, 
                full_output.pixels_per_point,
                self.is_playing, 
                self.camera.position, 
                self.camera.get_view_matrix(),
                self.ambient_color, 
                self.ambient_intensity, 
                &self.global_lights,
                &active_spots, 
                &active_points, 
                &visible_objects,
                &self.skybox_config, 
                &self.cloud_config, 
                &self.river_config, 
                &self.fog_config, 
                &self.smoke_emitter, 
                &self.fire_emitter, 
                &self.weather_emitter,
                current_time, 
            ) { 
                tracing::error!("Draw error: {}", e); 
            }

            // 4. Stifle CPU loop if hardware frame pacing is required to prevent overheating
            if frame_config::LIMIT_FRAMES {
                let elapsed = self.last_frame_time.elapsed();
                let target = Duration::from_secs_f64(1.0 / frame_config::TARGET_FPS as f64);
                if elapsed < target { 
                    std::thread::sleep(target - elapsed); 
                }
            }
            self.last_frame_time = Instant::now();
        }
    }
}