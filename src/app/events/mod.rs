// src/app/events/mod.rs

// This file simply exposes the internal modules that make up the Event Handler.
// By breaking it down, it is much easier to extend the game loop or add new inputs.

pub mod handler;
pub mod lifecycle;
pub mod input;
pub mod render_loop;