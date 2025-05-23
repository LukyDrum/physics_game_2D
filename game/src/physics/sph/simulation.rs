use std::collections::LinkedList;

use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::game::GameConfig;
use crate::math::Vector2;
use crate::physics::rigidbody::{BodyBehaviour, BodyForceAccumulation, RigidBody};
use crate::{physics::sph::Particle, utility::LookUp};

const PRESSURE_BASE: f32 = 100_000.0;
const BODY_COLLISION_FORCE_BASE: f32 = 10_000.0;

const PARTICLE_COLLIDER_RADIUS: f32 = 5.0;

fn kernel(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (1.0 - dist / radius).max(0.0).powi(2) * (3.0 / radius)
}

fn kernel_derivative(dist: f32, radius: f32) -> f32 {
    if dist > radius {
        return 0.0;
    }

    (6.0 * (dist - radius)) / radius.powi(2)
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
    mass: f32,
    sph_density: f32,
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

            p.sph_density = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &self.density_intermediates[*index];
                    if p.id == other_inter.id {
                        0.0
                    } else {
                        let (other_pos, other_mass) =
                            (other_inter.predicted_position, other_inter.mass);
                        let dist = (p.predicted_position - other_pos).length();
                        let density = other_mass * kernel(dist, self.smoothing_radius);
                        density
                    }
                })
                .sum();
        });
    }

    fn apply_pressures(&mut self) {
        self.particles
            .par_iter()
            .map(|p| PressureIntermediateReadOnly {
                predicted_position: p.predicted_position,
                pressure: p.pressure() * self.pressure_base,
                mass: p.mass(),
                sph_density: p.sph_density,
                id: p.id,
            })
            .collect_into_vec(&mut self.pressure_intermediates);

        self.particles.par_iter_mut().for_each(|p| {
            let pos = p.predicted_position;
            let pressure = p.pressure() * self.pressure_base;

            let neighbors = self.lookup.get_immediate_neighbors(&pos);
            let pressure_force: Vector2<f32> = neighbors
                .iter()
                .map(|index| {
                    let other_inter = &self.pressure_intermediates[*index];

                    if other_inter.sph_density == 0.0 || p.id == other_inter.id {
                        Vector2::zero()
                    } else {
                        let other_pressure = other_inter.pressure;
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
                        dir * other_inter.mass * shared_pressure
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
        bodies: &Vec<RigidBody>,
    ) -> Vec<(usize, BodyForceAccumulation)> {
        let mut body_forces = Vec::with_capacity(bodies.len());
        for (index, body) in bodies.iter().enumerate() {
            let force_accumulation = self
                .particles
                .par_iter_mut()
                .filter_map(|p| {
                    let circle = RigidBody::new_circle(
                        p.position,
                        PARTICLE_COLLIDER_RADIUS,
                        BodyBehaviour::Dynamic,
                    );

                    if let Some(collision_data) = RigidBody::check_collision(body, &circle) {
                        let elasticity = 0.3;
                        let impulse = -(1.0 + elasticity) * p.velocity.dot(collision_data.normal);
                        let impulse = impulse / (1.0 / p.mass() + 1.0 / body.state().mass());

                        p.velocity += collision_data.normal * (impulse / p.mass());
                        p.position += collision_data.normal * collision_data.penetration;

                        // Calculate force on body only for non-static bodies
                        if body.state().behaviour != BodyBehaviour::Static {
                            let mut force_accumulation = BodyForceAccumulation::empty();
                            let radius = collision_data.collision_points[0] - body.state().position;
                            let magnitude = -impulse
                                * p.body_collision_force_multiplier
                                * self.body_collision_base;
                            let force = collision_data.normal * magnitude;
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
        bodies: &Vec<RigidBody>,
        config: &GameConfig,
        dt: f32,
    ) -> Vec<(usize, BodyForceAccumulation)> {
        self.setup_lookup();

        self.gravity = config.gravity;
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
        self.resolve_collisions(bodies)
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

    /// Clears all particles = deletes all fluid in simulation
    pub fn clear_all_particles(&mut self) {
        self.particles.clear();
        self.lookup.clear();
        self.id_counter = 0;
    }
}
