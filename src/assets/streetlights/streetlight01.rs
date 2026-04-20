use crate::geometrical_shapes::{cylinder::Cylinder, game_object::GameObject, transform::Transform};
use crate::light::spot::SpotLight;
use glam::Vec3;

pub fn spawn() -> GameObject {
    // Generate a tall, thin grey pole
    let mesh = Cylinder::generate(0.2, 10.0, 12, [0.3, 0.3, 0.3, 1.0]);
    
    // Attach a bright yellow spotlight aiming straight down
    let spotlight = SpotLight {
        direction: Vec3::new(0.0, -1.0, 0.0), // Aiming DOWN
        color: [1.0, 0.9, 0.5],               // Warm yellow
        intensity: 5.0,
        range: 20.0,
        inner_cone_angle: 15.0_f32.to_radians(),
        outer_cone_angle: 30.0_f32.to_radians(),
        cast_shadows: true,
    };

    GameObject::new(mesh, Transform::default()).with_spot_light(spotlight)
}