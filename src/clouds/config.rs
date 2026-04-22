// src/clouds/config.rs
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct CloudConfig {
    pub base_color: [f32; 3],
    pub highlight_color: [f32; 3],
    pub cloud_coverage: f32,    // 0.0 = clear sky, 1.0 = overcast
    pub cloud_density: f32,     // How thick/dark the clouds are
    pub wind_speed: f32,        // Speed of clouds moving across the sky
    pub wind_direction: Vec3,   // Which way the wind blows
    pub height_min: f32,        // The bottom flat layer of the clouds
    pub height_max: f32,        // The fluffy top peaks of the clouds
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            base_color: [0.5, 0.6, 0.7],       // Darker ambient shadow inside the cloud
            highlight_color: [1.0, 1.0, 1.0],  // Sunlit edges
            cloud_coverage: 0.45,
            cloud_density: 0.8,
            wind_speed: 15.0,
            wind_direction: Vec3::new(1.0, 0.0, 0.5).normalize(),
            height_min: 150.0,
            height_max: 300.0,
        }
    }
}