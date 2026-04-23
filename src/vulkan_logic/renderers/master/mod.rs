// src/vulkan_logic/renderers/master/mod.rs
// Cleaned up the unused `DrawCall` import warning

pub mod renderer;
pub mod data_prep;
pub mod draw;
pub mod shadow_pass;
pub mod main_pass;
pub mod sync_submit;

// We export the struct itself from the renderer file rather than causing an unused import alias
pub use renderer::MasterRenderer;