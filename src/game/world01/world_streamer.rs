// src/game/world01/world_streamer.rs

use glam::Vec3;
use crate::assets::model::Model;
use crate::assets::gltf_loader::load_gltf_asset;
use crate::lights::spawnable::{spot::SpotLight, point::PointLight};

// Integrate the new dynamic spawnable array structs
use crate::water::config::RiverConfig;
use crate::smoke::emitter::SmokeEmitter;
use crate::smoke::config::SmokeConfig;
use crate::fire::emitter::FireEmitter;
use crate::fire::config::FireConfig;

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

/// The spatial boundary. If the camera crosses inside, all of these localized dynamic
/// arrays are merged into the main render pass automatically.
pub struct VirtualCuboid {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    
    // Spawnable Rendering Arrays
    pub assets_to_load: Vec<AssetDefinition>,
    pub spot_lights: Vec<SpotLight>,
    pub point_lights: Vec<PointLight>,
    
    // NEW: Fully integrated AAA spawnable VFX Arrays
    pub rivers: Vec<RiverConfig>,
    pub smoke_emitters: Vec<SmokeEmitter>,
    pub fire_emitters: Vec<FireEmitter>,
    
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
            spot_lights: vec![
                SpotLight::new(Vec3::new(8.0, 9.0, 5.0), Vec3::new(0.0, -1.0, 0.0), [1.0, 0.9, 0.5], 20.0, 200.0, 15.0, 30.0, true),
            ],
            point_lights: vec![
                PointLight::new(Vec3::new(0.0, 1.0, 3.0), [1.0, 0.0, 0.0], 5.0, 15.0),
            ],
            
            // Build and configure multiple localized effects inside the Zone!
            rivers: vec![
                RiverConfig {
                    position: Vec3::new(0.0, -1.5, 0.0),
                    ..Default::default()
                }
            ],
            smoke_emitters: vec![
                SmokeEmitter::new(SmokeConfig { position: Vec3::new(5.0, 4.0, -5.0), ..Default::default() })
            ],
            fire_emitters: vec![
                FireEmitter::new(FireConfig { position: Vec3::new(-2.0, -1.0, 3.0), ..Default::default() })
            ],
            
            is_loaded: false,
            loaded_models: Vec::new(),
        };

        Self { zones: vec![zone_01] }
    }

    /// Process physics exclusively for loaded emitters so we save CPU cycles
    pub fn tick(&mut self, dt: f32) {
        for zone in &mut self.zones {
            if zone.is_loaded {
                for smoke in &mut zone.smoke_emitters { smoke.tick(dt); }
                for fire in &mut zone.fire_emitters { fire.tick(dt); }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_visible_objects(&mut self, camera_pos: Vec3) -> (Vec<Model>, Vec<SpotLight>, Vec<PointLight>, Vec<RiverConfig>, Vec<SmokeEmitter>, Vec<FireEmitter>) {
        let mut models = Vec::new();
        let mut spots = Vec::new();
        let mut points = Vec::new();
        
        let mut rivers = Vec::new();
        let mut smokes = Vec::new();
        let mut fires = Vec::new();

        for zone in &mut self.zones {
            if zone.contains(camera_pos) {
                if !zone.is_loaded { zone.load_into_ram(); }
                
                // Copy references over to the active rendering loop
                models.extend(zone.loaded_models.clone());
                spots.extend(zone.spot_lights.clone());
                points.extend(zone.point_lights.clone());
                
                rivers.extend(zone.rivers.clone());
                smokes.extend(zone.smoke_emitters.clone());
                fires.extend(zone.fire_emitters.clone());
                
            } else if zone.is_loaded {
                zone.unload_from_ram();
            }
        }

        (models, spots, points, rivers, smokes, fires)
    }
}