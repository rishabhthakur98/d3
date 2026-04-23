// src/app/events/input.rs

use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{PhysicalKey, KeyCode};
use winit::window::CursorGrabMode;
use std::time::Instant;

use crate::app::engine_state::EngineApp;
use crate::game::menu::EngineAction;
use crate::{game, water};

impl EngineApp {
    // --------------------------------------------------------------------------------
    // Intercepts raw, unaccelerated mouse movement (Perfect for 3D Camera mapping)
    // --------------------------------------------------------------------------------
    pub(crate) fn handle_device_event(&mut self, event: DeviceEvent) {
        if self.is_playing {
            if let DeviceEvent::MouseMotion { delta } = event {
                game::world01::controls::handle_mouse(&mut self.camera, delta.0, delta.1);
            }
        }
    }

    // --------------------------------------------------------------------------------
    // The main router for Window Events (UI overlays, key bindings, resizing)
    // --------------------------------------------------------------------------------
    pub(crate) fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        // Prevent game actions if the user clicked inside an Egui window
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

    // --------------------------------------------------------------------------------
    // Maps keybinds directly to Player Actions or UI Menus
    // --------------------------------------------------------------------------------
    fn process_keyboard(&mut self, event_loop: &ActiveEventLoop, keycode: KeyCode, is_pressed: bool) {
        if self.is_playing {
            // Check for game exit (e.g. Escape button hit)
            if let EngineAction::ReturnToMenu = game::world01::controls::handle_keyboard(&mut self.input_state, keycode, is_pressed) {
                self.is_playing = false;
                self.menu.state = game::menu::MenuState::Main;
                
                // Release the mouse so the user can click options again
                if let Some(window) = &self.window { 
                    let _ = window.set_cursor_grab(CursorGrabMode::None); 
                    window.set_cursor_visible(true); 
                }
            }
        } else if is_pressed {
            // Manage UI State machines and Scene Loads
            match self.menu.handle_input(keycode) {
                EngineAction::Quit => event_loop.exit(),
                EngineAction::LoadWorld(world_name) => {
                    if world_name == "WORLD01" {
                        self.is_playing = true;
                        
                        // Restart physical trackers
                        self.camera = game::world01::camera::FreeformCamera::default();
                        self.input_state = game::world01::controls::InputState::default();
                        self.last_update_time = Instant::now(); 
                        
                        // Initialize local scene atmospheric states
                        self.ambient_color = game::world01::ambient_light_config::AMBIENT_LIGHT_COLOR;
                        self.ambient_intensity = game::world01::ambient_light_config::AMBIENT_LIGHT_INTENSITY;
                        self.global_lights = game::world01::ambient_light_config::get_global_lights();
                        self.skybox_config = game::world01::skybox_config::get_skybox_config();
                        
                        // Initialize physical props
                        let mut river = water::config::RiverConfig::default();
                        river.position = glam::Vec3::new(0.0, -1.5, 0.0);
                        self.river_config = river;

                        let mut smoke_cfg = crate::smoke::config::SmokeConfig::default();
                        smoke_cfg.position = glam::Vec3::new(5.0, 4.0, -5.0); 
                        self.smoke_emitter = crate::smoke::emitter::SmokeEmitter::new(smoke_cfg);

                        let mut fire_cfg = crate::fire::config::FireConfig::default();
                        fire_cfg.position = glam::Vec3::new(-2.0, -1.0, 3.0); 
                        self.fire_emitter = crate::fire::emitter::FireEmitter::new(fire_cfg);

                        self.weather_emitter.set_weather(crate::weather::config::WeatherConfig::heavy_rain());

                        // Lock the mouse to perfectly control the 3D camera
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