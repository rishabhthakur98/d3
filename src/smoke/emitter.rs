// src/smoke/emitter.rs
use super::config::SmokeConfig;
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub scale: f32,
    pub alpha: f32,
}

// A zero-dependency pseudo-random number generator for particle velocity
// Added Clone and Debug derives to allow the parent SmokeEmitter to be cloned
#[derive(Clone, Debug)] 
struct Lcg { state: u32 }

impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

/// The CPU simulation that calculates particle physics every frame
/// Added Clone derive so the WorldStreamer can hand off arrays safely to the Renderer
#[derive(Clone)] 
pub struct SmokeEmitter {
    pub config: SmokeConfig,
    pub particles: Vec<Particle>,
    accumulator: f32,
    rng: Lcg,
}

impl SmokeEmitter {
    pub fn new(config: SmokeConfig) -> Self {
        Self { config, particles: Vec::new(), accumulator: 0.0, rng: Lcg::new(42) }
    }

    pub fn tick(&mut self, dt: f32) {
        // 1. Update living particles
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.life -= dt;
            let progress = 1.0 - (p.life / p.max_life).clamp(0.0, 1.0);
            
            // Smoke expands over time
            p.scale = self.config.start_scale + (self.config.end_scale - self.config.start_scale) * progress;
            
            // Fade in quickly, fade out slowly
            p.alpha = if progress < 0.1 { progress * 10.0 } else { 1.0 - progress };
        }
        
        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);

        // 2. Spawn new particles based on spawn rate
        if self.config.spawn_rate > 0.0 {
            self.accumulator += dt;
            let spawn_interval = 1.0 / self.config.spawn_rate;

            while self.accumulator > spawn_interval {
                self.accumulator -= spawn_interval;
                if self.particles.len() < 500 { // Max 500 to fit safely in UBO bounds
                    let vx = (self.rng.next_f32() - 0.5) * self.config.spread;
                    let vz = (self.rng.next_f32() - 0.5) * self.config.spread;
                    
                    self.particles.push(Particle {
                        position: self.config.position,
                        velocity: Vec3::new(vx, self.config.rise_speed, vz),
                        life: self.config.particle_lifetime,
                        max_life: self.config.particle_lifetime,
                        scale: self.config.start_scale,
                        alpha: 0.0,
                    });
                }
            }
        }
    }
}

impl Default for SmokeEmitter {
    fn default() -> Self { Self::new(SmokeConfig::default()) }
}