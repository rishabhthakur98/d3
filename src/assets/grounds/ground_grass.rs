// src/assets/grounds/ground_grass.rs
// FIXED IMPORTS
use crate::geometrical_shapes::{rectangle::Rectangle, game_object::GameObject, transform::Transform};

pub fn spawn() -> GameObject {
    GameObject::new(Rectangle::generate(20.0, 20.0, [0.2, 0.8, 0.2, 1.0]), Transform::default())
}