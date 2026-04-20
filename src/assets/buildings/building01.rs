// src/assets/buildings/building01.rs
// FIXED IMPORTS
use crate::geometrical_shapes::{cuboid::Cuboid, game_object::GameObject, transform::Transform};

pub fn spawn() -> GameObject {
    GameObject::new(Cuboid::generate(10.0, 10.0, 10.0, [0.5, 0.5, 0.5, 1.0]), Transform::default())
}