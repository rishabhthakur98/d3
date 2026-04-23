// src/vulkan_logic/core/swapchain/mod.rs

// This folder safely encapsulates the heavily verbose Vulkan display initialization.

pub mod manager;
pub mod depth;
pub mod render_pass;
pub mod framebuffers;

// Expose only the manager struct to the rest of the application
pub use manager::SwapchainManager;