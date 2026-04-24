// src/weather/emitter.rs
use super::config::{WeatherConfig, WeatherType};
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct WeatherParticle {
    pub local_pos: Vec3, 
    pub random_seed: f32, 
}

#[derive(Clone, Debug)]
struct Lcg { state: u32 }
impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

#[derive(Clone)]
pub struct WeatherEmitter {
    pub config: WeatherConfig,
    pub particles: Vec<WeatherParticle>,
    #[allow(dead_code)] 
    rng: Lcg,
}

impl WeatherEmitter {
    pub fn new(config: WeatherConfig) -> Self {
        let mut rng = Lcg::new(12345);
        let mut particles = Vec::with_capacity(config.particle_count);

        for _ in 0..config.particle_count {
            particles.push(WeatherParticle {
                local_pos: Vec3::new(
                    (rng.next_f32() - 0.5) * config.box_size.x,
                    (rng.next_f32() - 0.5) * config.box_size.y,
                    (rng.next_f32() - 0.5) * config.box_size.z,
                ),
                random_seed: rng.next_f32() * std::f32::consts::TAU, 
            });
        }

        Self { config, particles, rng }
    }

    #[allow(dead_code)]
    pub fn set_weather(&mut self, config: WeatherConfig) {
        self.config = config;
        self.particles.clear();
        for _ in 0..self.config.particle_count {
            self.particles.push(WeatherParticle {
                local_pos: Vec3::new(
                    (self.rng.next_f32() - 0.5) * self.config.box_size.x,
                    (self.rng.next_f32() - 0.5) * self.config.box_size.y,
                    (self.rng.next_f32() - 0.5) * self.config.box_size.z,
                ),
                random_seed: self.rng.next_f32() * std::f32::consts::TAU,
            });
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.config.weather_type == WeatherType::None { return; }

        let half_y = self.config.box_size.y * 0.5;
        let half_x = self.config.box_size.x * 0.5;
        let half_z = self.config.box_size.z * 0.5;

        for p in &mut self.particles {
            p.local_pos.y -= self.config.fall_speed * dt;
            p.local_pos.x += self.config.wind_velocity.x * dt;
            p.local_pos.z += self.config.wind_velocity.z * dt;

            if p.local_pos.y < -half_y { p.local_pos.y += self.config.box_size.y; }
            if p.local_pos.x > half_x { p.local_pos.x -= self.config.box_size.x; }
            else if p.local_pos.x < -half_x { p.local_pos.x += self.config.box_size.x; }
            if p.local_pos.z > half_z { p.local_pos.z -= self.config.box_size.z; }
            else if p.local_pos.z < -half_z { p.local_pos.z += self.config.box_size.z; }
        }
    }
}

impl Default for WeatherEmitter {
    fn default() -> Self { Self::new(WeatherConfig::default()) }
}