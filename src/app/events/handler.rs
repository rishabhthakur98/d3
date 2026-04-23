// src/app/events/handler.rs

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent}; 
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId; 

use crate::app::engine_state::EngineApp;

// --------------------------------------------------------------------------------
// The core routing center for winit. Instead of housing the logic directly,
// this block simply delegates the events to highly-specialized helper methods.
// --------------------------------------------------------------------------------
impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_resumed(event_loop);
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        self.handle_device_event(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        self.handle_window_event(event_loop, event);
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.handle_about_to_wait();
    }
}