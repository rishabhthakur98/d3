// src/game/world01/mapsegment02.rs

use crate::assets::{grounds, buildings};
use crate::geometrical_shapes::game_object::GameObject; // FIXED IMPORT
use crate::geometrical_shapes::transform::Transform;    // NEEDED FOR CLOSURE

pub fn load_segment() -> Vec<GameObject> {
    let mut objects = Vec::new();

    objects.push(grounds::ground_sand::spawn().transform(
        |t: Transform| t.with_translation(20.0, -1.0, 0.0) // FIXED E0282
    ));

    objects.push(buildings::building03::spawn().transform(
        |t: Transform| t.with_translation(20.0, 0.0, 0.0)
             .with_rotation_euler(0.0, 45.0_f32.to_radians(), 0.0)
             .with_scale(1.0, 1.5, 1.0)
    ));

    objects
}