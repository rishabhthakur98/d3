// src/game/world01/mapsegment01.rs
use crate::assets::{grounds, buildings, streetlights}; 
use crate::geometrical_shapes::game_object::GameObject; 
use crate::geometrical_shapes::transform::Transform;    
use crate::light::spot::SpotLight;

pub fn load_segment() -> (Vec<GameObject>, Vec<SpotLight>) {
    let mut objects: Vec<GameObject> = Vec::new();
    let mut spots: Vec<SpotLight> = Vec::new();

    objects.push(grounds::ground_grass::spawn().transform(|t| t.with_translation(0.0, -1.0, 0.0)));
    objects.push(buildings::building01::spawn().transform(|t| t.with_translation(-5.0, 0.0, 0.0).with_scale(1.0, 2.0, 1.0)));
    objects.push(buildings::building02::spawn().transform(|t| t.with_translation(5.0, 0.0, -5.0)));

    // Extract both the physical pole and the light
    let (mut pole, mut light) = streetlights::streetlight01::spawn();
    
    // Move the physical pole
    pole = pole.transform(|t| t.with_translation(8.0, 4.0, 5.0));
    
    // Independent Light positioning:
    // The pole is at Y=4.0, and it is 10 units tall. So the top is at roughly Y=9.0
    light.position = glam::Vec3::new(8.0, 9.0, 5.0); 

    objects.push(pole);
    spots.push(light); // Track the light separately!

    (objects, spots)
}