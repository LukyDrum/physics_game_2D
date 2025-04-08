mod particle;
mod simulation;

use game_macros::UIEditable;
use {
    crate::math::Vector2,
    particle::{BODY_COLLISION_FORCE_BASE, PRESSURE_BASE},
};
pub use {particle::Particle, simulation::Sph};
