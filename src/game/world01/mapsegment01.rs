// src/game/world01/mapsegment01.rs

use crate::assets::{grounds, buildings, streetlights}; 
use crate::geometrical_shapes::game_object::GameObject; 
use crate::light::{spot::SpotLight, point::PointLight};
use glam::Vec3;

pub fn load_segment() -> (Vec<GameObject>, Vec<SpotLight>, Vec<PointLight>) {
    let mut objects: Vec<GameObject> = Vec::new();
    let mut spots: Vec<SpotLight> = Vec::new();
    let mut points: Vec<PointLight> = Vec::new();

    objects.push(grounds::ground_grass::spawn().transform(|t| t.with_translation(0.0, -1.0, 0.0)));
    
    objects.push(buildings::building01::spawn().transform(|t| t.with_translation(-5.0, 0.0, 0.0).with_scale(1.0, 2.0, 1.0)));
    objects.push(buildings::building02::spawn().transform(|t| t.with_translation(5.0, 0.0, -5.0)));

    // 1. Unpack the Streetlight (Geometry + Spotlight)
    let (mut pole, mut light) = streetlights::streetlight01::spawn();
    pole = pole.transform(|t| t.with_translation(8.0, 4.0, 5.0));
    light.position = Vec3::new(8.0, 9.0, 5.0); 
    
    objects.push(pole);
    spots.push(light);

    // 2. NEW: Add a standalone glowing Point Light in the middle of the grass
    let red_beacon = PointLight {
        position: Vec3::new(0.0, 1.0, 3.0),
        color: [1.0, 0.0, 0.0], // Pure Red
        intensity: 5.0,         // Bright
        range: 15.0,            // Shines outward for 15 units
    };
    points.push(red_beacon);

    (objects, spots, points)
}