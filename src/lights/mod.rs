// src/lights/mod.rs

// The master module for all lighting systems, logically segregated into 
// physical spawnable entities, global atmospheric entities, and GPU core structs.
pub mod spawnable;
pub mod global;
pub mod core;