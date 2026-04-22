// src/fire/emitter.rs
use super::config::FireConfig;
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct FireParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub scale: f32,
    pub color: [f32; 3],
    pub alpha: f32,
}

// Pseudo-random number generator for randomizing flame flicker
struct Lcg { state: u32 }
impl Lcg {
    fn new(seed: u32) -> Self { Self { state: seed } }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

pub struct FireEmitter {
    pub config: FireConfig,
    pub particles: Vec<FireParticle>,
    accumulator: f32,
    rng: Lcg,
}

impl FireEmitter {
    pub fn new(config: FireConfig) -> Self {
        Self { config, particles: Vec::new(), accumulator: 0.0, rng: Lcg::new(84) }
    }

    pub fn tick(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.position += p.velocity * dt;
            p.life -= dt;
            let progress = 1.0 - (p.life / p.max_life).clamp(0.0, 1.0);
            
            // Fire shrinks as it burns out
            p.scale = self.config.start_scale + (self.config.end_scale - self.config.start_scale) * progress;
            
            // Fire fades from yellow/white core to deep red edges
            p.color = [
                self.config.core_color[0] + (self.config.edge_color[0] - self.config.core_color[0]) * progress,
                self.config.core_color[1] + (self.config.edge_color[1] - self.config.core_color[1]) * progress,
                self.config.core_color[2] + (self.config.edge_color[2] - self.config.core_color[2]) * progress,
            ];
            
            // Fire fades out at the very tip
            p.alpha = if progress > 0.6 { 1.0 - ((progress - 0.6) / 0.4) } else { 1.0 };
        }
        
        self.particles.retain(|p| p.life > 0.0);

        if self.config.spawn_rate > 0.0 {
            self.accumulator += dt;
            let spawn_interval = 1.0 / self.config.spawn_rate;

            while self.accumulator > spawn_interval {
                self.accumulator -= spawn_interval;
                if self.particles.len() < 500 { 
                    // Fire flickers inward
                    let vx = (self.rng.next_f32() - 0.5) * self.config.spread;
                    let vz = (self.rng.next_f32() - 0.5) * self.config.spread;
                    
                    self.particles.push(FireParticle {
                        position: self.config.position + Vec3::new(vx, 0.0, vz),
                        velocity: Vec3::new(vx * -0.5, self.config.rise_speed * (0.8 + self.rng.next_f32() * 0.4), vz * -0.5), // Pulls inward slightly as it rises
                        life: self.config.particle_lifetime,
                        max_life: self.config.particle_lifetime,
                        scale: self.config.start_scale,
                        color: self.config.core_color,
                        alpha: 1.0,
                    });
                }
            }
        }
    }
}

impl Default for FireEmitter {
    fn default() -> Self { Self::new(FireConfig::default()) }
}