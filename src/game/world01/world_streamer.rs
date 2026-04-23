// src/game/world01/world_streamer.rs

use glam::Vec3;
use crate::assets::model::Model;
use crate::assets::gltf_loader::load_gltf_asset;
use crate::light::{spot::SpotLight, point::PointLight};

/// Configuration representing a specific 3D mesh that lives on the hard drive
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

/// A spatial boundary. If the camera crosses inside, the disk is hit to load assets to RAM.
/// If the camera leaves, the models are purged from RAM immediately.
pub struct VirtualCuboid {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub assets_to_load: Vec<AssetDefinition>,
    
    // We can also bind highly-localized light sources specifically to this space
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
        tracing::info!("VirtualCuboid entered. Assets loaded to RAM.");
    }

    pub fn unload_from_ram(&mut self) {
        if !self.is_loaded { return; }
        
        // Let Rust's borrow checker automatically wipe the memory
        self.loaded_models.clear();
        self.is_loaded = false;
        tracing::info!("VirtualCuboid exited. Assets purged from RAM.");
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
        // --- Configure Level Boundaries ---
        let zone_01 = VirtualCuboid {
            min_bounds: Vec3::new(-200.0, -100.0, -200.0),
            max_bounds: Vec3::new(200.0, 100.0, 200.0),
            assets_to_load: vec![
                AssetDefinition::new("assets/models/terrain.glb", Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
            ],
            spot_lights: Vec::new(),
            point_lights: Vec::new(),
            is_loaded: false,
            loaded_models: Vec::new(),
        };

        Self { zones: vec![zone_01] }
    }

    /// Evaluates the physical coordinates of the player and streams data dynamically
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