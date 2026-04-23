// src/game/world01/world_streamer.rs
use glam::Vec3;
use crate::assets::model::Model;
use crate::assets::gltf_loader::load_gltf_asset;
use crate::lights::spawnable::{spot::SpotLight, point::PointLight};
use crate::water::config::RiverConfig;
use crate::smoke::emitter::SmokeEmitter;
use crate::smoke::config::SmokeConfig;
use crate::fire::emitter::FireEmitter;
use crate::fire::config::FireConfig;
use crate::weather::emitter::WeatherEmitter;
use crate::weather::config::WeatherConfig;
use crate::volumetrics::fog_config::FogVolume;
use crate::clouds::config::CloudVolume; // NEW

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
    
    pub assets_to_load: Vec<AssetDefinition>,
    pub spot_lights: Vec<SpotLight>,
    pub point_lights: Vec<PointLight>,
    
    pub rivers: Vec<RiverConfig>,
    pub smoke_emitters: Vec<SmokeEmitter>,
    pub fire_emitters: Vec<FireEmitter>,
    pub weather_emitters: Vec<WeatherEmitter>, 
    pub fog_volumes: Vec<FogVolume>,
    pub cloud_volumes: Vec<CloudVolume>, // NEW
    
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
            
            assets_to_load: vec![AssetDefinition::new("assets/models/terrain.glb", Vec3::ZERO, Vec3::ZERO, Vec3::ONE)],
            spot_lights: vec![SpotLight::new(Vec3::new(8.0, 9.0, 5.0), Vec3::new(0.0, -1.0, 0.0), [1.0, 0.9, 0.5], 20.0, 200.0, 15.0, 30.0, true)],
            point_lights: vec![PointLight::new(Vec3::new(0.0, 1.0, 3.0), [1.0, 0.0, 0.0], 5.0, 15.0)],
            
            rivers: vec![RiverConfig { position: Vec3::new(0.0, -1.5, 0.0), ..Default::default() }],
            smoke_emitters: vec![SmokeEmitter::new(SmokeConfig { position: Vec3::new(5.0, 4.0, -5.0), ..Default::default() })],
            fire_emitters: vec![FireEmitter::new(FireConfig { position: Vec3::new(-2.0, -1.0, 3.0), ..Default::default() })],
            weather_emitters: vec![WeatherEmitter::new(WeatherConfig::heavy_rain())],
            
            fog_volumes: vec![FogVolume::new(Vec3::new(-50.0, -10.0, -50.0), Vec3::new(50.0, 20.0, 50.0), [0.6, 0.7, 0.8], 0.025)],
            
            // NEW: Clouds are now localized! Covers from Y=150 to Y=300 locally!
            cloud_volumes: vec![CloudVolume::new(
                Vec3::new(-200.0, 150.0, -200.0), 
                Vec3::new(200.0, 300.0, 200.0)
            )],
            
            is_loaded: false,
            loaded_models: Vec::new(),
        };

        Self { zones: vec![zone_01] }
    }

    pub fn tick(&mut self, dt: f32) {
        for zone in &mut self.zones {
            if zone.is_loaded {
                for smoke in &mut zone.smoke_emitters { smoke.tick(dt); }
                for fire in &mut zone.fire_emitters { fire.tick(dt); }
                for weather in &mut zone.weather_emitters { weather.tick(dt); }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn get_visible_objects(&mut self, camera_pos: Vec3) -> (Vec<Model>, Vec<SpotLight>, Vec<PointLight>, Vec<RiverConfig>, Vec<SmokeEmitter>, Vec<FireEmitter>, Vec<WeatherEmitter>, Vec<FogVolume>, Vec<CloudVolume>) {
        let mut models = Vec::new();
        let mut spots = Vec::new();
        let mut points = Vec::new();
        let mut rivers = Vec::new();
        let mut smokes = Vec::new();
        let mut fires = Vec::new();
        let mut weathers = Vec::new();
        let mut fogs = Vec::new();
        let mut clouds = Vec::new(); // NEW

        for zone in &mut self.zones {
            if zone.contains(camera_pos) {
                if !zone.is_loaded { zone.load_into_ram(); }
                
                models.extend(zone.loaded_models.clone());
                spots.extend(zone.spot_lights.clone());
                points.extend(zone.point_lights.clone());
                rivers.extend(zone.rivers.clone());
                smokes.extend(zone.smoke_emitters.clone());
                fires.extend(zone.fire_emitters.clone());
                weathers.extend(zone.weather_emitters.clone());
                fogs.extend(zone.fog_volumes.clone());
                clouds.extend(zone.cloud_volumes.clone()); // NEW
                
            } else if zone.is_loaded {
                zone.unload_from_ram();
            }
        }

        (models, spots, points, rivers, smokes, fires, weathers, fogs, clouds)
    }
}