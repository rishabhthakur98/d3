// src/game/world01/world_streamer.rs
use glam::Vec3;
use std::sync::mpsc::{self, Receiver};
use std::thread;

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
use crate::clouds::config::CloudVolume;

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
    pub cloud_volumes: Vec<CloudVolume>,
    
    // --- Asynchronous Loading State ---
    pub is_loaded: bool,
    pub is_loading: bool,
    pub model_receiver: Option<Receiver<Vec<Model>>>,
    pub loaded_models: Vec<Model>,
}

impl VirtualCuboid {
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min_bounds.x && point.x <= self.max_bounds.x &&
        point.y >= self.min_bounds.y && point.y <= self.max_bounds.y &&
        point.z >= self.min_bounds.z && point.z <= self.max_bounds.z
    }

    /// Spawns a background thread to handle heavy disk I/O and parsing without freezing the game loop.
    pub fn load_into_ram(&mut self) {
        if self.is_loaded || self.is_loading { return; }
        self.is_loading = true;

        // Establish a thread-safe communication channel
        let (tx, rx) = mpsc::channel();
        self.model_receiver = Some(rx);

        // Clone the lightweight asset definitions to hand off to the thread
        let assets = self.assets_to_load.clone();

        thread::spawn(move || {
            let mut models = Vec::new();
            for asset in assets {
                match load_gltf_asset(&asset.file_path, asset.location, asset.orientation, asset.scale) {
                    Ok(model) => models.push(model),
                    Err(e) => tracing::error!("Background Streamer Error loading {}: {}", asset.file_path, e),
                }
            }
            
            // Send the completed models back to the main thread.
            // We intentionally ignore the result here; if the player leaves the zone before 
            // loading finishes, the main thread will drop the Receiver, making this fail gracefully.
            let _ = tx.send(models);
        });
    }

    /// Safely polls the background thread to see if the assets are ready for rendering.
    pub fn check_loading(&mut self) {
        if self.is_loading {
            if let Some(rx) = &self.model_receiver {
                match rx.try_recv() {
                    Ok(models) => {
                        // The thread successfully delivered the loaded geometry!
                        self.loaded_models = models;
                        self.is_loaded = true;
                        self.is_loading = false;
                        self.model_receiver = None;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // Thread is still working, do nothing and let the game keep playing smoothly.
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // The thread panicked or died unexpectedly.
                        tracing::error!("Background loading thread disconnected unexpectedly.");
                        self.is_loading = false;
                        self.model_receiver = None;
                    }
                }
            }
        }
    }

    pub fn unload_from_ram(&mut self) {
        if !self.is_loaded && !self.is_loading { return; }
        
        self.loaded_models.clear();
        self.is_loaded = false;
        self.is_loading = false;
        
        // Setting the receiver to None drops it. If the background thread is currently
        // running, its `tx.send(models)` will safely return an Error and drop the models without crashing.
        self.model_receiver = None; 
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
            weather_emitters: vec![WeatherEmitter::new(WeatherConfig::default())],
            
            fog_volumes: vec![FogVolume::new(Vec3::new(-50.0, -10.0, -50.0), Vec3::new(50.0, 20.0, 50.0), [0.6, 0.7, 0.8], 0.025)],
            
            cloud_volumes: vec![CloudVolume::new(
                Vec3::new(-200.0, 150.0, -200.0), 
                Vec3::new(200.0, 300.0, 200.0)
            )],
            
            is_loaded: false,
            is_loading: false,
            model_receiver: None,
            loaded_models: Vec::new(),
        };

        Self { zones: vec![zone_01] }
    }

    pub fn tick(&mut self, dt: f32) {
        for zone in &mut self.zones {
            // Keep particles simulating even if the underlying terrain is still downloading in the background
            if zone.is_loaded || zone.is_loading {
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
        let mut clouds = Vec::new(); 

        for zone in &mut self.zones {
            // Continuously verify background thread status
            zone.check_loading();

            if zone.contains(camera_pos) {
                // If not loaded and not currently trying to load, trigger the background pipeline
                if !zone.is_loaded && !zone.is_loading { 
                    zone.load_into_ram(); 
                }
                
                // Models only get rendered once the thread delivers them
                if zone.is_loaded {
                    models.extend(zone.loaded_models.clone());
                }
                
                // Procedural entities (Lights, Rivers, Particles, Fog) are lightweight.
                // We render them immediately so the player isn't in a totally empty void while the thread works!
                spots.extend(zone.spot_lights.clone());
                points.extend(zone.point_lights.clone());
                rivers.extend(zone.rivers.clone());
                smokes.extend(zone.smoke_emitters.clone());
                fires.extend(zone.fire_emitters.clone());
                weathers.extend(zone.weather_emitters.clone());
                fogs.extend(zone.fog_volumes.clone());
                clouds.extend(zone.cloud_volumes.clone()); 
                
            } else if zone.is_loaded || zone.is_loading {
                zone.unload_from_ram();
            }
        }

        (models, spots, points, rivers, smokes, fires, weathers, fogs, clouds)
    }
}