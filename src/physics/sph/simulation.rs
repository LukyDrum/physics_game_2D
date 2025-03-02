use macroquad::math::Vec2;
use rayon::iter::{ParallelIterator, IntoParallelRefMutIterator};

use crate::{utility::LookUp, physics::sph::Particle};

pub struct SimulationConfig {
    pub gravity: Vec2,
    pub collision_damping: f32,
    pub smoothing_radius: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function simulating default
    pub const fn default() -> Self {
        SimulationConfig {
            gravity: Vec2::new(0.0, 9.8),
            collision_damping: 0.2,
            smoothing_radius: 20.0,
        }
    }
}

fn kernel(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (1.0 - dist / radius).max(0.0).powi(2)
}

fn near_kernel(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (1.0 - dist / radius).max(0.0).powi(3)
}

fn kernel_derivative(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (2.0 * (dist - radius)) / (radius.powi(2))
}

fn near_kernel_derivative(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (3.0 * (dist - radius)) / (radius.powi(3))
}

pub struct Sph {
    pub particles: Vec<Particle>,
    pub lookup: LookUp,
    pub gravity: Vec2,
    pub smoothing_radius: f32,
}

impl Sph {
    pub fn new(config: SimulationConfig, width: f32, height: f32) -> Self {
        Sph {
            particles: Vec::new(),
            lookup: LookUp::new(width, height, config.smoothing_radius),
            gravity: config.gravity,
            smoothing_radius: config.smoothing_radius,
        }
    }

    fn calculate_densities(&mut self) {
        for i in 0..self.particles.len() {
            let pos = self.particles[i].predicted_position;

            let neighbors = self.lookup.get_immediate_neighbors(pos);
            (self.particles[i].sph_density, self.particles[i].sph_near_density) = neighbors
                .iter()
                .map(|index| {
                    let p = &self.particles[*index];
                    let dist = (pos - p.predicted_position).length();
                    let density = p.mass() * kernel(dist, self.smoothing_radius);
                    let near_density = p.mass() * near_kernel(dist, self.smoothing_radius);
                    (density, near_density)
                })
                .fold((0.0, 0.0), |acc, e| (acc.0 + e.0, acc.1 + e.1));
        }
    }

    fn apply_pressures(&mut self) {
        for i in 0..self.particles.len() {
            let pos = self.particles[i].predicted_position;
            let pressure = self.particles[i].pressure();
            let near_pressure = self.particles[i].near_pressure();

            let neighbors = self.lookup.get_immediate_neighbors(pos);
            let pressure_force: Vec2 = neighbors
                .iter()
                .map(|index| {
                    let p = &self.particles[*index];
                    let pos_diff = p.predicted_position - pos;
                    let dir = pos_diff.normalize();
                    let other_pressure = p.pressure();
                    let other_near_pressure = p.near_pressure();

                    if dir.is_nan() || p.sph_density == 0.0 {
                        Vec2::ZERO
                    } else {
                        let dist = pos_diff.length();
                        let shared_pressure = (pressure + other_pressure) / (2.0 * p.sph_density)
                            * kernel_derivative(dist, self.smoothing_radius);
                        let shared_near_pressure = (near_pressure + other_near_pressure)
                            / (2.0 * p.sph_near_density)
                            * near_kernel_derivative(dist, self.smoothing_radius);
                        p.mass() * (shared_pressure + shared_near_pressure) * dir
                    }
                })
                .sum();

            self.particles[i].add_force(pressure_force);
        }
    }

    fn setup_lookup(&mut self) {
        self.lookup.clear();
        for index in 0..self.particles.len() {
            self.lookup.insert(&self.particles[index], index);
        }
    }

    pub fn step(&mut self, delta_time: f32) {
        self.setup_lookup();

        let dt = delta_time;

        self.particles
            .par_iter_mut()
            .for_each(|p| p.predict_position(dt));
        self.calculate_densities();
        self.apply_pressures();
        self.particles.par_iter_mut().for_each(|p| {
            p.add_force(p.mass() * self.gravity);
            p.apply_accumulated_force(dt);
            p.move_by_velocity(dt);
        });
    }
}


