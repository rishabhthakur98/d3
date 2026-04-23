// src/game/world01/world_streamer.rs
// Updated to support dynamically spawning 100s of highly customizable lights

use glam::Vec3;
use crate::assets::model::Model;
use crate::assets::gltf_loader::load_gltf_asset;
use crate::lights::spawnable::{spot::SpotLight, point::PointLight};

#[derive(Clone, Debug)]
pub struct AssetDefinition {
    pub file_path: String,
    pub location: Vec3,
    pub orientation: Vec3, 
    pub scale: Vec3,
}

impl AssetDefinition {
    pub fn new(file_path: &str, location: Vec3, orientation: Vec3, scale: Vec3) -> Self {
        Self { file_path: file_path.to_string(), location, orientation, scale }
    }
}

pub struct VirtualCuboid {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    
    // Spawnable Arrays
    pub assets_to_load: Vec<AssetDefinition>,
    pub spot_lights: Vec<SpotLight>,
    pub point_lights: Vec<PointLight>,
    
    pub is_loaded: bool,
    pub loaded_models: Vec<Model>,
}

impl VirtualCuboid {
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min_bounds.x && point.x <= self.max_bounds.x &&
        point.y >= self.min_bounds.y && point.y <= self.max_bounds.y &&
        point.z >= self.min_bounds.z && point.z <= self.max_bounds.z
    }

    pub fn load_into_ram(&mut self) {
        if self.is_loaded { return; }
        
        self.loaded_models.clear();
        for asset in &self.assets_to_load {
            match load_gltf_asset(&asset.file_path, asset.location, asset.orientation, asset.scale) {
                Ok(model) => self.loaded_models.push(model),
                Err(e) => tracing::error!("Streamer Error: {}", e),
            }
        }
        
        self.is_loaded = true;
    }

    pub fn unload_from_ram(&mut self) {
        if !self.is_loaded { return; }
        self.loaded_models.clear();
        self.is_loaded = false;
    }
}

pub struct WorldStreamer {
    pub zones: Vec<VirtualCuboid>,
}

impl Default for WorldStreamer {
    fn default() -> Self { Self::new() }
}

impl WorldStreamer {
    pub fn new() -> Self {
        let zone_01 = VirtualCuboid {
            min_bounds: Vec3::new(-200.0, -100.0, -200.0),
            max_bounds: Vec3::new(200.0, 100.0, 200.0),
            
            assets_to_load: vec![
                AssetDefinition::new("assets/models/terrain.glb", Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
            ],
            
            // Highly customized, builder-pattern localized lights!
            spot_lights: vec![
                SpotLight::new(
                    Vec3::new(8.0, 9.0, 5.0),    // Position
                    Vec3::new(0.0, -1.0, 0.0),   // Pointing straight down
                    [1.0, 0.9, 0.5],             // Warm yellowish color
                    20.0,                        // Intensity
                    200.0,                       // Distance range
                    15.0,                        // Core inner cone
                    30.0,                        // Fading outer cone
                    true                         // Generate shadow map?
                ),
            ],
            
            point_lights: vec![
                PointLight::new(
                    Vec3::new(0.0, 1.0, 3.0),    // Position
                    [1.0, 0.0, 0.0],             // Pure Red
                    5.0,                         // Intensity
                    15.0                         // Distance range
                ),
            ],
            
            is_loaded: false,
            loaded_models: Vec::new(),
        };

        Self { zones: vec![zone_01] }
    }

    pub fn get_visible_objects(&mut self, camera_pos: Vec3) -> (Vec<Model>, Vec<SpotLight>, Vec<PointLight>) {
        let mut active_models = Vec::new();
        let mut active_spots = Vec::new();
        let mut active_points = Vec::new();

        for zone in &mut self.zones {
            if zone.contains(camera_pos) {
                if !zone.is_loaded { zone.load_into_ram(); }
                
                active_models.extend(zone.loaded_models.clone());
                active_spots.extend(zone.spot_lights.clone());
                active_points.extend(zone.point_lights.clone());
            } else if zone.is_loaded {
                zone.unload_from_ram();
            }
        }

        (active_models, active_spots, active_points)
    }
}