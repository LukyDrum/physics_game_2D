use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::game::{GameBody, GameConfig};
use crate::math::{v2, Vector2};
use crate::physics::rigidbody::{BodyBehaviour, BodyForceAccumulation};
use crate::{physics::sph::Particle, utility::LookUp};

use super::particle::{BODY_COLLISION_FORCE_BASE, PRESSURE_BASE};

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
    pressure_base: f32,
    body_collision_base: f32,

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
            pressure_base: PRESSURE_BASE,
            body_collision_base: BODY_COLLISION_FORCE_BASE,

            id_counter: 0,
            // 1000 chosen as a good starting capacity
            density_intermediates: Vec::with_capacity(1000),
            pressure_intermediates: Vec::with_capacity(1000),
        }
    }

    pub fn particle_count(&self) -> usize {
        self.particles.len()
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
                pressure: p.pressure() * self.pressure_base,
                near_pressure: p.near_pressure(),
                mass: p.mass(),
                sph_density: p.sph_density,
                sph_near_density: p.sph_near_density,
                id: p.id,
            })
            .collect_into_vec(&mut self.pressure_intermediates);

        self.particles.par_iter_mut().for_each(|p| {
            let pos = p.predicted_position;
            let pressure = p.pressure() * self.pressure_base;
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

    /// Resolves collision for the particles and calculates acumulated forces that act on the
    /// bodies.
    fn resolve_collisions(
        &mut self,
        bodies: &Vec<Box<dyn GameBody>>,
        delta_time: f32,
    ) -> Vec<(usize, BodyForceAccumulation)> {
        let mut body_forces = Vec::with_capacity(bodies.len());
        for (index, body) in bodies.iter().enumerate() {
            let force_accumulation = self
                .particles
                .par_iter_mut()
                .filter_map(|p| {
                    if body.contains_point(p.position) {
                        // Use particles position before moving to resolve collisions.
                        // The actual point of contact should be very close to the middle between those
                        // 2 positions.
                        let collision_info =
                            body.point_collision_data(p.position - p.velocity * delta_time * 0.5);
                        let elasticity = 0.3;
                        let impulse =
                            -(1.0 + elasticity) * p.velocity.dot(collision_info.surface_normal);
                        let impulse = impulse / (1.0 / p.mass() + 1.0 / body.state().mass());

                        p.velocity += collision_info.surface_normal * (impulse / p.mass());
                        p.position = collision_info.surface_point;

                        // Calculate force on body only for non-static bodies
                        if body.state().behaviour != BodyBehaviour::Static {
                            let mut force_accumulation = BodyForceAccumulation::empty();
                            let radius = collision_info.surface_point - body.state().position;
                            let magnitude = -impulse
                                * p.body_collision_force_multiplier
                                * self.body_collision_base;
                            let force = collision_info.surface_normal * magnitude;
                            force_accumulation.add_force_at_radius(force, radius);

                            Some(force_accumulation)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .reduce(
                    || BodyForceAccumulation::empty(),
                    |a, b| BodyForceAccumulation {
                        force: a.force + b.force,
                        torque: a.torque + b.torque,
                    },
                );

            body_forces.push((index, force_accumulation));
        }

        body_forces
    }

    fn setup_lookup(&mut self) {
        self.lookup.clear();
        for index in 0..self.particles.len() {
            self.lookup
                .insert(&self.particles[index].predicted_position, index);
        }
    }

    /// Performs a step of the fluid simulation.
    /// At the end of the step, it resolves any collisions with the provided bodies and returns the
    /// forces that the fluid exerts on the bodies.
    pub fn step(
        &mut self,
        bodies: &Vec<Box<dyn GameBody>>,
        config: &GameConfig,
        dt: f32,
    ) -> Vec<(usize, BodyForceAccumulation)> {
        self.setup_lookup();

        self.gravity = config.sph_config.gravity;
        self.pressure_base = config.sph_config.base_pressure;
        self.body_collision_base = config.sph_config.base_body_force;

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
        self.resolve_collisions(bodies, dt)
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
