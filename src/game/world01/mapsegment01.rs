use crate::assets::{grounds, buildings, streetlights}; // Imported streetlights!
use crate::geometrical_shapes::game_object::GameObject; 
use crate::geometrical_shapes::transform::Transform;    

pub fn load_segment() -> Vec<GameObject> {
    let mut objects: Vec<GameObject> = Vec::new();

    objects.push(grounds::ground_grass::spawn().transform(
        |t: Transform| t.with_translation(0.0, -1.0, 0.0) 
    ));

    objects.push(buildings::building01::spawn().transform(
        |t: Transform| t.with_translation(-5.0, 0.0, 0.0).with_scale(1.0, 2.0, 1.0)
    ));

    objects.push(buildings::building02::spawn().transform(
        |t: Transform| t.with_translation(5.0, 0.0, -5.0)
    ));

    // NEW: Spawn the streetlight and translate it up so the base is on the ground
    objects.push(streetlights::streetlight01::spawn().transform(
        |t: Transform| t.with_translation(8.0, 4.0, 5.0) 
    ));

    objects
}