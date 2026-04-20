use crate::geometrical_shapes::mesh::Mesh;
use crate::geometrical_shapes::transform::Transform;

/// The fundamental building block of your game world.
#[derive(Clone)]
pub struct GameObject {
    pub mesh: Mesh,
    pub transform: Transform,
}

impl GameObject {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Self { mesh, transform }
    }

    /// Builder method to easily apply position/rotation/scale inline.
    /// This fixes the compiler error you saw in mapsegment01!
    pub fn transform<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(Transform) -> Transform 
    {
        self.transform = f(self.transform);
        self
    }
}