// src/geometrical_shapes/game_object.rs
use crate::geometrical_shapes::mesh::Mesh;
use crate::geometrical_shapes::transform::Transform;

/// A physical object in the 3D world.
/// Notice how it NO LONGER holds any light data! It is purely for rendering geometry.
#[derive(Clone)]
pub struct GameObject {
    pub mesh: Mesh,
    pub transform: Transform,
}

impl GameObject {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Self { mesh, transform }
    }

    pub fn transform<F>(mut self, f: F) -> Self where F: FnOnce(Transform) -> Transform {
        self.transform = f(self.transform);
        self
    }
}