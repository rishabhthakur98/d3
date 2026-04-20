use crate::geometrical_shapes::mesh::Mesh;
use crate::geometrical_shapes::transform::Transform;

// Import our new lights!
use crate::light::{ambient::AmbientLight, global::GlobalLight, point::PointLight, spot::SpotLight};

#[derive(Clone)]
pub struct GameObject {
    pub mesh: Mesh,
    pub transform: Transform,
    
    // Optional light components attached to this physical object
    pub ambient_light: Option<AmbientLight>,
    pub global_light: Option<GlobalLight>,
    pub point_light: Option<PointLight>,
    pub spot_light: Option<SpotLight>,
}

impl GameObject {
    pub fn new(mesh: Mesh, transform: Transform) -> Self {
        Self { 
            mesh, transform, 
            ambient_light: None, global_light: None, point_light: None, spot_light: None 
        }
    }

    pub fn transform<F>(mut self, f: F) -> Self where F: FnOnce(Transform) -> Transform {
        self.transform = f(self.transform);
        self
    }

    // Builder methods to easily attach lights
    pub fn with_spot_light(mut self, light: SpotLight) -> Self {
        self.spot_light = Some(light);
        self
    }
}