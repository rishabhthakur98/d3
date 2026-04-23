// src/vulkan_logic/renderers/master/mod.rs

// This file simply exposes the internal modules that make up the Master Renderer.
// By breaking it down, we make the codebase vastly easier to navigate and maintain.

pub mod renderer;
pub mod draw;
pub mod data_prep;
pub mod shadow_pass;
pub mod main_pass;
pub mod sync_submit;

// Expose the main struct to the rest of the application so `app/engine_state.rs`
// doesn't have to change its import paths.
pub use renderer::MasterRenderer;

// Expose the internal DrawCall struct strictly to the submodules within `master/`
pub(crate) use renderer::DrawCall;