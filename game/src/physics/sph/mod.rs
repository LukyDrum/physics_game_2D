mod particle;
mod simulation;

use {
    crate::math::Vector2,
    particle::{BODY_COLLISION_FORCE_BASE, PRESSURE_BASE},
};
pub use {particle::Particle, simulation::Sph};

/// Values for configuring the SPH fluid simulation.
#[derive(Clone)]
pub struct SphConfig {
    /// Base pressure multiplier for each particle. Individual values are computed using this and
    /// the particles mass.
    pub base_pressure: f32,
    /// Similiar to `base_pressure` but only affects the particles effect on rigidbodies.
    pub base_body_force: f32,
    /// The force of gravity acting on the fluid.
    pub gravity: Vector2<f32>,
}

impl Default for SphConfig {
    fn default() -> Self {
        SphConfig {
            base_pressure: PRESSURE_BASE,
            base_body_force: BODY_COLLISION_FORCE_BASE,
            gravity: Vector2::new(0.0, 981.0),
        }
    }
}
