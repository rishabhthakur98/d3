// src/app/events/lifecycle.rs

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use std::sync::Arc;

use crate::app::engine_state::EngineApp;
use crate::vulkan_logic::core::context::VulkanContext;
use crate::vulkan_logic::renderers::master::MasterRenderer;

impl EngineApp {
    // --------------------------------------------------------------------------------
    // Handles initial boot sequences, OS context connections, and creating
    // the heavy Vulkan allocations and windowing constraints.
    // --------------------------------------------------------------------------------
    pub(crate) fn handle_resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("D3 Engine")
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))); 
            
            if let Ok(window) = event_loop.create_window(window_attributes) {
                let window_arc = Arc::new(window);
                self.window = Some(window_arc.clone());
                
                // Mount UI state specifically to the window bounds
                let scale_factor = window_arc.scale_factor() as f32;
                self.egui_state = Some(egui_winit::State::new(
                    self.egui_ctx.clone(), 
                    egui::ViewportId::ROOT, 
                    window_arc.as_ref(), 
                    Some(scale_factor), 
                    None, 
                    None, 
                ));
                
                // Initialize massive GPU subsystems
                match VulkanContext::new(window_arc.as_ref()) {
                    Ok(ctx) => {
                        let ctx_arc = Arc::new(ctx);
                        self.context = Some(ctx_arc.clone());
                        
                        match MasterRenderer::new(ctx_arc, window_arc.as_ref()) {
                            Ok(renderer) => self.renderer = Some(renderer),
                            Err(e) => { 
                                tracing::error!("Failed to init Renderer: {:?}", e); 
                                event_loop.exit(); 
                            }
                        }
                    }
                    Err(e) => { 
                        tracing::error!("Failed to init Vulkan: {:?}", e); 
                        event_loop.exit(); 
                    }
                }
            }
        }
    }

    // --------------------------------------------------------------------------------
    // Fired right before the event loop idles. Triggers continuous rendering loops.
    // --------------------------------------------------------------------------------
    pub(crate) fn handle_about_to_wait(&mut self) {
        if self.renderer.is_some() { 
            if let Some(window) = &self.window { 
                window.request_redraw(); 
            } 
        }
    }
}