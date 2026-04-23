// src/app/events/input.rs

use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::CursorGrabMode;
use std::time::Instant;

use crate::app::engine_state::EngineApp;
use crate::game::menu::EngineAction;
use crate::game;

impl EngineApp {
    pub(crate) fn handle_device_event(&mut self, event: DeviceEvent) {
        if self.is_playing {
            if let DeviceEvent::MouseMotion { delta } = event {
                game::world01::controls::handle_mouse(&mut self.camera, delta.0, delta.1);
            }
        }
    }

    pub(crate) fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
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
                if let Some(r) = &mut self.renderer { r.is_resized = true; } 
            },
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(keycode), state, .. }, .. } => {
                let is_pressed = state == ElementState::Pressed;
                self.process_keyboard(event_loop, keycode, is_pressed);
            }
            WindowEvent::RedrawRequested => {
                self.process_redraw();
            }
            _ => {}
        }
    }

    fn process_keyboard(&mut self, event_loop: &ActiveEventLoop, keycode: KeyCode, is_pressed: bool) {
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
                        self.global_lights = game::world01::global_light_config::get_global_lights();
                        self.skybox_config = game::world01::skybox_config::get_skybox_config();
                        
                        self.weather_emitter.set_weather(crate::weather::config::WeatherConfig::heavy_rain());

                        if let Some(w) = &self.window { 
                            let _ = w.set_cursor_grab(CursorGrabMode::Confined)
                                     .or_else(|_| w.set_cursor_grab(CursorGrabMode::Locked)); 
                            w.set_cursor_visible(false); 
                        }
                    }
                }
                _ => {}
            }
        }
        
        if let Some(window) = &self.window { 
            window.request_redraw(); 
        }
    }
}