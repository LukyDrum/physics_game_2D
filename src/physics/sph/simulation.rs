use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::game::GameBody;
use crate::math::{v2, Vector2};
use crate::{physics::sph::Particle, utility::LookUp};

fn kernel(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (1.0 - dist / radius).max(0.0).powi(2)
}

const NEAR_MAX: f32 = 10_000.0;

fn near_kernel(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }
    if dist == 0.0 {
        return NEAR_MAX;
    }

    let radius_inv = 1.0 / radius;
    (radius_inv / dist - radius_inv).max(NEAR_MAX)
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
    if dist == 0.0 {
        return -NEAR_MAX;
    }

    (-1.0 / radius * dist.powi(2)).max(-NEAR_MAX)
}

/// This a helper structure which references fields from the `Particle` struct.
/// Using this enables us to parallelize the calculation of densities.
/// For clarity they are named the same as in the `Particle` struct
///
/// Contains read only fields needed for density calculations.
struct DensityIntermediateReadOnly {
    predicted_position: Vector2<f32>,
    mass: f32,
    id: u32,
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
    id: u32,
}

pub struct Sph {
    pub particles: Vec<Particle>,
    pub lookup: LookUp<usize>,
    pub gravity: Vector2<f32>,
    pub smoothing_radius: f32,

    // Inner helping stuff
    id_counter: u32,
    density_intermediates: Vec<DensityIntermediateReadOnly>,
    pressure_intermediates: Vec<PressureIntermediateReadOnly>,
}

impl Sph {
    pub fn new(width: f32, height: f32) -> Self {
        let smoothing_radius = 12.0;
        Sph {
            particles: Vec::new(),
            lookup: LookUp::new(width, height, smoothing_radius * 2.0),
            gravity: Vector2::new(0.0, 981.0),
            smoothing_radius,

            id_counter: 0,
            // 1000 chosen as a good starting capacity
            density_intermediates: Vec::with_capacity(1000),
            pressure_intermediates: Vec::with_capacity(1000),
        }
    }

    pub fn add_particle(&mut self, mut particle: Particle) {
        // Add very small offset to particles position
        particle.position += v2!(fastrand::f32() - 0.5, fastrand::f32() - 0.5);
        let pos = particle.position;

        particle.id = self.id_counter;
        self.particles.push(particle);
        self.id_counter += 1;

        // Insert particles index into lookup
        let index = self.particles.len() - 1;
        self.lookup.insert(&pos, index);
    }

    fn add_gravity_force(&mut self) {
        self.particles
            .par_iter_mut()
            .for_each(|p| p.add_force(self.gravity * p.mass));
    }

    fn calculate_densities(&mut self) {
        // Get readonly fields of the particles needed for density calculation.
        self.particles
            .par_iter()
            .map(|p| DensityIntermediateReadOnly {
                predicted_position: p.predicted_position,
                mass: p.mass(),
                id: p.id,
            })
            .collect_into_vec(&mut self.density_intermediates);

        self.particles.par_iter_mut().for_each(|p| {
            let neighbors = self.lookup.get_immediate_neighbors(&p.predicted_position);

            (p.sph_density, p.sph_near_density) = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &self.density_intermediates[*index];
                    if p.id == other_inter.id {
                        (0.0, 0.0)
                    } else {
                        let (other_pos, other_mass) =
                            (other_inter.predicted_position, other_inter.mass);
                        let dist = (p.predicted_position - other_pos).length();
                        let density = other_mass * kernel(dist, self.smoothing_radius);
                        let near_density = other_mass * near_kernel(dist, self.smoothing_radius);
                        (density, near_density)
                    }
                })
                .fold((0.0, 0.0), |acc, e| (acc.0 + e.0, acc.1 + e.1));
        });
    }

    fn apply_pressures(&mut self) {
        self.particles
            .par_iter()
            .map(|p| PressureIntermediateReadOnly {
                predicted_position: p.predicted_position,
                pressure: p.pressure(),
                near_pressure: p.near_pressure(),
                mass: p.mass(),
                sph_density: p.sph_density,
                sph_near_density: p.sph_near_density,
                id: p.id,
            })
            .collect_into_vec(&mut self.pressure_intermediates);

        self.particles.par_iter_mut().for_each(|p| {
            let pos = p.predicted_position;
            let pressure = p.pressure();
            let near_pressure = p.near_pressure();

            let neighbors = self.lookup.get_immediate_neighbors(&pos);
            let pressure_force: Vector2<f32> = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &self.pressure_intermediates[*index];

                    if other_inter.sph_density == 0.0 || p.id == other_inter.id {
                        Vector2::zero()
                    } else {
                        let other_pressure = other_inter.pressure;
                        let other_near_pressure = other_inter.near_pressure;
                        let pos_diff = other_inter.predicted_position - pos;

                        let dir = if pos_diff.is_zero() {
                            Vector2::<f32>::random_unit()
                        } else {
                            pos_diff.normalized()
                        };
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

    fn resolve_collisions(&mut self, bodies: &Vec<Box<dyn GameBody>>) {
        self.particles.par_iter_mut().for_each(|p| {
            for body in bodies {
                if body.contains_point(p.position) {
                    // Move particle to surface
                    let collision_info = body.point_collision_data(p.position);
                    let elasticity = 0.1;
                    let impulse =
                        -(1.0 + elasticity) * p.velocity.dot(collision_info.surface_normal);
                    // Let's say that body has mass of 100
                    let impulse = impulse / (1.0 / p.mass() + 1.0 / 100.0);

                    p.velocity += collision_info.surface_normal * (impulse / p.mass());
                    p.position = collision_info.surface_point;
                }
            }
        });
    }

    fn setup_lookup(&mut self) {
        self.lookup.clear();
        for index in 0..self.particles.len() {
            self.lookup
                .insert(&self.particles[index].predicted_position, index);
        }
    }

    pub fn step(&mut self, delta_time: f32, bodies: &Vec<Box<dyn GameBody>>) {
        self.setup_lookup();

        let dt = delta_time;

        self.particles
            .par_iter_mut()
            .for_each(|p| p.predict_position(dt));
        // Add gravity force
        self.add_gravity_force();
        self.calculate_densities();
        self.apply_pressures();
        // Apply accumulated force and move particle by it
        self.particles.par_iter_mut().for_each(|p| {
            p.apply_accumulated_force(dt);
            p.move_by_velocity(dt);
        });

        // Do collision detection and resolution
        self.resolve_collisions(bodies);
        self.particles.par_iter_mut().for_each(|p| {
            p.move_by_velocity(dt);
        });
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
