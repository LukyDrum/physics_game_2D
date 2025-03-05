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
/// Contains only read only fields needed for density calculations.
/// More info at `[DensityIntermediateMutable]`
struct DensityIntermediateReadOnly {
    predicted_position: Vector2<f32>,
    mass: f32,
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
                mass: p.mass,
            })
            .collect_into_vec(&mut intermediates_read_only);

        self.particles.par_iter_mut().for_each(|p| {
            let neighbors = self
                .lookup
                .get_immediate_neighbors(&p.predicted_position);

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
        for i in 0..self.particles.len() {
            let pos = self.particles[i].predicted_position;
            let pressure = self.particles[i].pressure();
            let near_pressure = self.particles[i].near_pressure();

            let neighbors = self.lookup.get_immediate_neighbors(&pos);
            let pressure_force: Vector2<f32> = neighbors
                .iter()
                .map(|index| {
                    let p = &self.particles[*index];
                    let pos_diff = p.predicted_position - pos;
                    let dir = pos_diff.normalized();
                    let other_pressure = p.pressure();
                    let other_near_pressure = p.near_pressure();

                    if dir.is_nan() || p.sph_density == 0.0 {
                        Vector2::zero()
                    } else {
                        let dist = pos_diff.length();
                        let shared_pressure = (pressure + other_pressure) / (2.0 * p.sph_density)
                            * kernel_derivative(dist, self.smoothing_radius);
                        let shared_near_pressure = (near_pressure + other_near_pressure)
                            / (2.0 * p.sph_near_density)
                            * near_kernel_derivative(dist, self.smoothing_radius);
                        dir * p.mass() * (shared_pressure + shared_near_pressure)
                    }
                })
                .sum();

            self.particles[i].add_force(pressure_force);
        }
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
}
