// src/app/event_handler.rs

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent}; 
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId, CursorGrabMode}; 
use std::sync::Arc;
use std::time::{Instant, Duration}; 

use super::engine_state::EngineApp;
use crate::game::menu::EngineAction;
use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::renderers::master::MasterRenderer;
use crate::{game, water, frame_config}; 

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("D3 Engine")
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))); 
            
            if let Ok(window) = event_loop.create_window(window_attributes) {
                let window_arc = Arc::new(window);
                self.window = Some(window_arc.clone());
                
                let scale_factor = window_arc.scale_factor() as f32;
                self.egui_state = Some(egui_winit::State::new(
                    self.egui_ctx.clone(), egui::ViewportId::ROOT, window_arc.as_ref(), Some(scale_factor), None, None, 
                ));
                
                match VulkanContext::new(window_arc.as_ref()) {
                    Ok(ctx) => {
                        let ctx_arc = Arc::new(ctx);
                        self.context = Some(ctx_arc.clone());
                        
                        match MasterRenderer::new(ctx_arc, window_arc.as_ref()) {
                            Ok(renderer) => self.renderer = Some(renderer),
                            Err(e) => { tracing::error!("Failed to init Renderer: {:?}", e); event_loop.exit(); }
                        }
                    }
                    Err(e) => { tracing::error!("Failed to init Vulkan: {:?}", e); event_loop.exit(); }
                }
            }
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: winit::event::DeviceId, event: DeviceEvent) {
        if self.is_playing {
            if let DeviceEvent::MouseMotion { delta } = event {
                game::world01::controls::handle_mouse(&mut self.camera, delta.0, delta.1);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(state), Some(window)) = (&mut self.egui_state, &self.window) {
            let res = state.on_window_event(window.as_ref(), &event);
            if !self.is_playing && res.consumed {
                if res.repaint { window.request_redraw(); }
                if !matches!(event, WindowEvent::Resized(_)) { return; }
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => { if let Some(r) = &mut self.renderer { r.is_resized = true; } }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(keycode), state, .. }, .. } => {
                let is_pressed = state == ElementState::Pressed;
                if self.is_playing {
                    if let EngineAction::ReturnToMenu = game::world01::controls::handle_keyboard(&mut self.input_state, keycode, is_pressed) {
                        self.is_playing = false;
                        self.menu.state = game::menu::MenuState::Main;
                        if let Some(window) = &self.window { let _ = window.set_cursor_grab(CursorGrabMode::None); window.set_cursor_visible(true); }
                    }
                } else if is_pressed {
                    match self.menu.handle_input(keycode) {
                        EngineAction::Quit => event_loop.exit(),
                        EngineAction::LoadWorld(world_name) => {
                            if world_name == "WORLD01" {
                                self.is_playing = true;
                                self.camera = game::world01::camera::FreeformCamera::default();
                                self.input_state = game::world01::controls::InputState::default();
                                self.last_update_time = Instant::now(); 
                                self.ambient_color = game::world01::ambient_light_config::AMBIENT_LIGHT_COLOR;
                                self.ambient_intensity = game::world01::ambient_light_config::AMBIENT_LIGHT_INTENSITY;
                                self.global_lights = game::world01::ambient_light_config::get_global_lights();
                                self.skybox_config = game::world01::skybox_config::get_skybox_config();
                                
                                let mut river = water::config::RiverConfig::default();
                                river.position = glam::Vec3::new(0.0, -1.5, 0.0);
                                self.river_config = river;

                                if let Some(w) = &self.window { let _ = w.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| w.set_cursor_grab(CursorGrabMode::Locked)); w.set_cursor_visible(false); }
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(window) = &self.window { window.request_redraw(); }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = (now - self.last_update_time).as_secs_f32();
                self.last_update_time = now; 
                let current_time = self.engine_start_time.elapsed().as_secs_f32(); 
                
                if let (Some(renderer), Some(window), Some(state)) = (&mut self.renderer, &self.window, &mut self.egui_state) {
                    let (visible_objects, active_spots, active_points) = if self.is_playing {
                        game::world01::controls::update_camera_position(&mut self.camera, &self.input_state, delta_time);
                        self.world_streamer.get_visible_objects(self.camera.position)
                    } else { (Vec::new(), Vec::new(), Vec::new()) };

                    let raw_input = state.take_egui_input(window.as_ref());
                    let full_output = if !self.is_playing {
                        let options = self.menu.get_current_options(); let cursor = self.menu.cursor_index;
                        self.egui_ctx.run(raw_input, |ctx| {
                            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(0, 0, 180))).show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(150.0); ui.heading(egui::RichText::new("D3 ENGINE").size(80.0).color(egui::Color32::WHITE)); ui.add_space(80.0);
                                    for (i, opt) in options.iter().enumerate() {
                                        let text = if i == cursor { format!("-> [ {} ] <-", opt) } else { opt.to_string() };
                                        let color = if i == cursor { egui::Color32::YELLOW } else { egui::Color32::GRAY };
                                        ui.label(egui::RichText::new(text).size(40.0).color(color)); ui.add_space(20.0);
                                    }
                                });
                            });
                        })
                    } else { self.egui_ctx.run(raw_input, |_| {}) };

                    state.handle_platform_output(window.as_ref(), full_output.platform_output);
                    let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                    if let Err(e) = renderer.draw_frame(
                        window.as_ref(), &clipped_primitives, &full_output.textures_delta, full_output.pixels_per_point,
                        self.is_playing, self.camera.position, self.camera.get_view_matrix(),
                        self.ambient_color, self.ambient_intensity, &self.global_lights,
                        &active_spots, &active_points, &visible_objects,
                        &self.skybox_config, &self.river_config, &self.fog_config, current_time, 
                    ) { tracing::error!("Draw error: {}", e); }

                    if frame_config::LIMIT_FRAMES {
                        let elapsed = self.last_frame_time.elapsed();
                        let target = Duration::from_secs_f64(1.0 / frame_config::TARGET_FPS as f64);
                        if elapsed < target { std::thread::sleep(target - elapsed); }
                    }
                    self.last_frame_time = Instant::now();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() { if let Some(window) = &self.window { window.request_redraw(); } }
    }
}