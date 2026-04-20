// src/game/world01/mapsegment02.rs
use crate::assets::{grounds, buildings};
use crate::geometrical_shapes::game_object::GameObject; 
use crate::geometrical_shapes::transform::Transform;    
use crate::light::spot::SpotLight;

pub fn load_segment() -> (Vec<GameObject>, Vec<SpotLight>) {
    let mut objects = Vec::new();
    let spots = Vec::new(); // Empty list, no lights in this segment

    objects.push(grounds::ground_sand::spawn().transform(|t| t.with_translation(20.0, -1.0, 0.0)));
    objects.push(buildings::building03::spawn().transform(
        |t| t.with_translation(20.0, 0.0, 0.0).with_rotation_euler(0.0, 45.0_f32.to_radians(), 0.0).with_scale(1.0, 1.5, 1.0)
    ));

    (objects, spots)
}