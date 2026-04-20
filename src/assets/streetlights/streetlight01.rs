// src/assets/streetlights/streetlight01.rs
use crate::geometrical_shapes::{cylinder::Cylinder, game_object::GameObject, transform::Transform};
use crate::light::spot::SpotLight;
use glam::Vec3;

/// Returns the physical pole AND the standalone light source
pub fn spawn() -> (GameObject, SpotLight) {
    let mesh = Cylinder::generate(0.2, 10.0, 12, [0.3, 0.3, 0.3, 1.0]);
    let pole = GameObject::new(mesh, Transform::default());
    
    // Note: The map segment will be responsible for syncing the position of the light
    // to the top of the physical pole.
    let spotlight = SpotLight {
        position: Vec3::ZERO, 
        direction: Vec3::new(0.0, -1.0, 0.0), 
        color: [1.0, 0.9, 0.5],               
        intensity: 20.0,
        range: 200.0,
        inner_cone_angle: 15.0_f32.to_radians(),
        outer_cone_angle: 30.0_f32.to_radians(),
        cast_shadows: true,
    };

    (pole, spotlight)
}