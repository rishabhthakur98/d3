// src/main.rs
mod app;
mod volumetrics;
mod water; 
mod skybox;
mod smoke; 
mod clouds; 
mod fire; 
mod assets;
mod geometrical_shapes;
mod backface_cull_config;
mod game;
mod light;
mod vulkan_logic;
mod frame_config; 

use winit::event_loop::{ControlFlow, EventLoop};
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut engine_app = app::engine_state::EngineApp::default();
    event_loop.run_app(&mut engine_app)?;
    
    Ok(())
}