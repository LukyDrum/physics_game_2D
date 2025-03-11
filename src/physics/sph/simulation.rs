use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::math::Vector2;
use crate::{physics::sph::Particle, utility::LookUp};

pub struct SimulationConfig {
    pub gravity: Vector2<f32>,
    pub collision_damping: f32,
    pub smoothing_radius: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function simulating default
    pub const fn default() -> Self {
        SimulationConfig {
            gravity: Vector2::new(0.0, 9.8),
            collision_damping: 0.2,
            smoothing_radius: 12.0,
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

/// This a helper structure which references fields from the `Particle` struct.
/// Using this enables us to parallelize the calculation of densities.
/// For clarity they are named the same as in the `Particle` struct
///
/// Contains read only fields needed for density calculations.
struct DensityIntermediateReadOnly {
    predicted_position: Vector2<f32>,
    mass: f32,
}

/// Contains read only fields needed for pressure calculations.
/// More info at `[DensityIntermediateReadOnly]`
struct PressureIntermediateReadOnly {
    predicted_position: Vector2<f32>,
    pressure: f32,
    near_pressure: f32,
    mass: f32,
    sph_density: f32,
    sph_near_density: f32,
}

pub struct Sph {
    pub particles: Vec<Particle>,
    pub lookup: LookUp<usize>,
    pub gravity: Vector2<f32>,
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

    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    fn calculate_densities(&mut self) {
        // Get readonly fields of the particles needed for density calculation.
        let mut intermediates_read_only = Vec::with_capacity(self.particles.len());
        self.particles
            .par_iter()
            .map(|p| DensityIntermediateReadOnly {
                predicted_position: p.predicted_position,
                mass: p.mass(),
            })
            .collect_into_vec(&mut intermediates_read_only);

        self.particles.par_iter_mut().for_each(|p| {
            let neighbors = self.lookup.get_immediate_neighbors(&p.predicted_position);

            (p.sph_density, p.sph_near_density) = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &intermediates_read_only[*index];
                    let (other_pos, other_mass) =
                        (other_inter.predicted_position, other_inter.mass);
                    let dist = (p.predicted_position - other_pos).length();
                    let density = other_mass * kernel(dist, self.smoothing_radius);
                    let near_density = other_mass * near_kernel(dist, self.smoothing_radius);
                    (density, near_density)
                })
                .fold((0.0, 0.0), |acc, e| (acc.0 + e.0, acc.1 + e.1));
        });
    }

    fn apply_pressures(&mut self) {
        let mut intermediates_read_only = Vec::with_capacity(self.particles.len());
        self.particles
            .par_iter()
            .map(|p| PressureIntermediateReadOnly {
                predicted_position: p.predicted_position,
                pressure: p.pressure(),
                near_pressure: p.near_pressure(),
                mass: p.mass(),
                sph_density: p.sph_density,
                sph_near_density: p.sph_near_density,
            })
            .collect_into_vec(&mut intermediates_read_only);

        self.particles.par_iter_mut().for_each(|p| {
            let pos = p.predicted_position;
            let pressure = p.pressure();
            let near_pressure = p.near_pressure();

            let neighbors = self.lookup.get_immediate_neighbors(&pos);
            let pressure_force: Vector2<f32> = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &intermediates_read_only[*index];
                    let pos_diff = other_inter.predicted_position - pos;
                    let dir = pos_diff.normalized();
                    let other_pressure = other_inter.pressure;
                    let other_near_pressure = other_inter.near_pressure;

                    if dir.is_nan() || other_inter.sph_density == 0.0 {
                        Vector2::zero()
                    } else {
                        let dist = pos_diff.length();
                        let shared_pressure = (pressure + other_pressure)
                            / (2.0 * other_inter.sph_density)
                            * kernel_derivative(dist, self.smoothing_radius);
                        let shared_near_pressure = (near_pressure + other_near_pressure)
                            / (2.0 * other_inter.sph_near_density)
                            * near_kernel_derivative(dist, self.smoothing_radius);
                        dir * other_inter.mass * (shared_pressure + shared_near_pressure)
                    }
                })
                .sum();

            p.add_force(pressure_force);
        });
    }

    fn setup_lookup(&mut self) {
        self.lookup.clear();
        for index in 0..self.particles.len() {
            self.lookup
                .insert(&self.particles[index].predicted_position, index);
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
            p.add_force(self.gravity * p.mass());
            p.apply_accumulated_force(dt);
            p.move_by_velocity(dt);
        });
    }

    /// Used for the marching squares algorithm.
    /// Returns the concentration of particles at the specified position.
    pub fn concentration_at_position(&self, position: Vector2<f32>, radius: f32) -> f32 {
        let neighbors = self.lookup.get_neighbors_in_radius(&position, radius);

        neighbors
            .iter()
            .map(|index| {
                let p_pos = self.particles[*index].position;
                let dist = (position - p_pos).length();
                radius / dist
            })
            .sum()
    }

    pub fn get_particles_around_position(
        &self,
        position: Vector2<f32>,
        radius: f32,
    ) -> LinkedList<&Particle> {
        let neighbors = self.lookup.get_neighbors_in_radius(&position, radius);

        neighbors
            .iter()
            .map(|index| &self.particles[*index])
            .collect()
    }
}
