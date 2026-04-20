// src/game/world01/mapsegment01.rs

use crate::assets::{grounds, buildings};
use crate::geometrical_shapes::game_object::GameObject; // FIXED IMPORT
use crate::geometrical_shapes::transform::Transform;    // NEEDED FOR CLOSURE

pub fn load_segment() -> Vec<GameObject> {
    let mut objects: Vec<GameObject> = Vec::new();

    objects.push(grounds::ground_grass::spawn().transform(
        |t: Transform| t.with_translation(0.0, -1.0, 0.0) // FIXED E0282: Explicitly defining t as Transform
    ));

    objects.push(buildings::building01::spawn().transform(
        |t: Transform| t.with_translation(-5.0, 0.0, 0.0).with_scale(1.0, 2.0, 1.0)
    ));

    objects.push(buildings::building02::spawn().transform(
        |t: Transform| t.with_translation(5.0, 0.0, -5.0)
    ));

    objects
}