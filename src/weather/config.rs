// src/weather/config.rs
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeatherType {
    None,
    Rain,
    Snow,
}

#[derive(Clone, Debug)]
pub struct WeatherConfig {
    pub weather_type: WeatherType,
    pub particle_count: usize, // Number of particles in the box
    pub box_size: Vec3,        // The dimensions of the invisible box around the camera
    pub fall_speed: f32,       // Base downward velocity
    pub wind_velocity: Vec3,   // Pushes the weather sideways
    pub particle_scale: f32,   // Size of the rain drops or snowflakes
    pub color: [f32; 4],       // Color and transparency
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            weather_type: WeatherType::None,
            particle_count: 2000, 
            box_size: Vec3::new(40.0, 40.0, 40.0), // 40-meter cube around the camera
            fall_speed: 0.0,
            wind_velocity: Vec3::ZERO,
            particle_scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl WeatherConfig {
    pub fn heavy_rain() -> Self {
        Self {
            weather_type: WeatherType::Rain,
            particle_count: 3000,
            box_size: Vec3::new(30.0, 40.0, 30.0),
            fall_speed: 25.0, // Falls very fast
            wind_velocity: Vec3::new(5.0, 0.0, 2.0),
            particle_scale: 0.15, // Thin drops
            color: [0.8, 0.85, 0.9, 0.6],
        }
    }

    pub fn gentle_snow() -> Self {
        Self {
            weather_type: WeatherType::Snow,
            particle_count: 1500,
            box_size: Vec3::new(30.0, 30.0, 30.0),
            fall_speed: 2.0, // Falls very slowly
            wind_velocity: Vec3::new(1.0, 0.0, 0.5),
            particle_scale: 0.25, // Large fluffy flakes
            color: [1.0, 1.0, 1.0, 0.8],
        }
    }
}