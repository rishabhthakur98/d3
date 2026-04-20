// src/main.rs
mod assets;
mod geometrical_shapes;
mod backface_cull_config;
mod game;
mod light;
mod vulkan_logic;
mod frame_config; 

use game::menu::{EngineAction, MenuSystem};
use vulkan_logic::context::VulkanContext;
use vulkan_logic::renderer::VulkanRenderer;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent}; 
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId, CursorGrabMode}; 
use std::sync::Arc;
use std::time::{Instant, Duration}; 
use anyhow::Result;

struct EngineApp {
    window: Option<Arc<Window>>,
    menu: MenuSystem,
    context: Option<Arc<VulkanContext>>,
    renderer: Option<VulkanRenderer>,
    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    
    is_playing: bool,
    input_state: game::world01::controls::InputState,
    world_streamer: game::world01::segmantload_wrt_camera::WorldStreamer,
    camera: game::world01::camera::FreeformCamera,
    
    ambient_color: [f32; 3],
    ambient_intensity: f32,
    global_lights: Vec<light::global::GlobalLight>,
    
    last_update_time: Instant, 
    last_frame_time: Instant,  
}

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
                
                let state = egui_winit::State::new(
                    self.egui_ctx.clone(), egui::ViewportId::ROOT, window_arc.as_ref(), Some(scale_factor), None, None, 
                );
                self.egui_state = Some(state);
                
                if let Ok(ctx) = VulkanContext::new(window_arc.as_ref()) {
                    let ctx_arc = Arc::new(ctx);
                    self.context = Some(ctx_arc.clone());
                    if let Ok(renderer) = VulkanRenderer::new(ctx_arc, window_arc.as_ref()) {
                        self.renderer = Some(renderer);
                    }
                }
            }
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
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
            WindowEvent::Resized(_) => {
                if let Some(renderer) = &mut self.renderer { renderer.is_resized = true; }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(keycode), state, .. }, .. } => {
                let is_pressed = state == ElementState::Pressed;

                if self.is_playing {
                    if let EngineAction::ReturnToMenu = game::world01::controls::handle_keyboard(&mut self.input_state, keycode, is_pressed) {
                        self.is_playing = false;
                        self.menu.state = game::menu::MenuState::Main;
                        if let Some(window) = &self.window {
                            let _ = window.set_cursor_grab(CursorGrabMode::None);
                            window.set_cursor_visible(true);
                        }
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
                                
                                self.global_lights.clear();
                                self.global_lights.push(light::global::GlobalLight {
                                    direction: glam::Vec3::from_array(game::world01::ambient_light_config::SUN_DIRECTION),
                                    color: game::world01::ambient_light_config::SUN_COLOR,
                                    intensity: game::world01::ambient_light_config::SUN_INTENSITY,
                                    cast_shadows: true, 
                                });
                                
                                if let Some(window) = &self.window {
                                    let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
                                    window.set_cursor_visible(false);
                                }
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
                
                if let (Some(renderer), Some(window), Some(state)) = (&mut self.renderer, &self.window, &mut self.egui_state) {
                    
                    // NEW: Unpack the independent mesh list AND the active lights list
                    let (visible_objects, active_spots) = if self.is_playing {
                        game::world01::controls::update_camera_position(&mut self.camera, &self.input_state, delta_time);
                        self.world_streamer.get_visible_objects(self.camera.position)
                    } else {
                        (Vec::new(), Vec::new())
                    };

                    let raw_input = state.take_egui_input(window.as_ref());
                    
                    let full_output = if !self.is_playing {
                        let options = self.menu.get_current_options();
                        let cursor = self.menu.cursor_index;
                        
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
                    } else {
                        self.egui_ctx.run(raw_input, |_ctx| {})
                    };

                    state.handle_platform_output(window.as_ref(), full_output.platform_output);
                    let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                    let camera_pos = self.camera.position;
                    let camera_view = self.camera.get_view_matrix();

                    if let Err(e) = renderer.draw_frame(
                        window.as_ref(), 
                        &clipped_primitives, 
                        &full_output.textures_delta, 
                        full_output.pixels_per_point,
                        self.is_playing,
                        camera_pos,
                        camera_view,
                        self.ambient_color,
                        self.ambient_intensity,
                        &self.global_lights,
                        &active_spots, // Pass the standalone lights!
                        &visible_objects
                    ) {
                        tracing::error!("Draw error: {}", e);
                    }

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
        if let Some(window) = &self.window { window.request_redraw(); }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EngineApp {
        window: None, menu: MenuSystem::new(), context: None, renderer: None,
        egui_ctx: egui::Context::default(), egui_state: None,
        
        is_playing: false, input_state: game::world01::controls::InputState::default(),
        world_streamer: game::world01::segmantload_wrt_camera::WorldStreamer::new(),
        camera: game::world01::camera::FreeformCamera::default(),
        
        ambient_color: [0.0, 0.0, 0.0], ambient_intensity: 0.0, global_lights: Vec::new(),
        
        last_update_time: Instant::now(), last_frame_time: Instant::now(),
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}