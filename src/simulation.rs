use macroquad::math::Vec2;

use crate::runge_kutta;

pub struct SimulationConfig {
    pub gravity: Vec2,
    pub collision_damping: f32,
    pub smoothing_radius: f32,
    pub pressure_multiplier: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function simulating default
    pub const fn default() -> Self {
        SimulationConfig {
            gravity: Vec2::new(0.0, 9.8),
            collision_damping: 0.2,
            smoothing_radius: 14.0,
            pressure_multiplier: 300.0,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub predicted_position: Vec2,
    pub velocity: Vec2,
    pub sph_density: f32,
    pub sph_near_density: f32,
    pub mass: f32,
    accumulated_force: Vec2,
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self::new_with_velocity(position, Vec2::ZERO)
    }

    pub fn new_with_velocity(position: Vec2, velocity: Vec2) -> Self {
        Particle {
            position,
            predicted_position: position,
            velocity,
            sph_density: 1.0,
            sph_near_density: 1.0,
            mass: 1.0,
            accumulated_force: Vec2::ZERO,
        }
    }

    /// Sets the accumulated force to a new value.
    /// Should not be use for simulations.
    pub fn set_force(&mut self, force: Vec2) {
        self.accumulated_force = force;
    }

    /// Adds `force` to the accumulated force.
    pub fn add_force(&mut self, force: Vec2) {
        self.accumulated_force += force;
    }

    pub fn apply_accumulated_force(&mut self, delta_time: f32) {
        if self.accumulated_force.length_squared() < 0.001 {
            return;
        }

        let acceleration = self.accumulated_force / self.mass;
        
        self.velocity = runge_kutta(self.velocity, delta_time, acceleration);
        // Reset the accumulated force
        self.accumulated_force = Vec2::ZERO;
    }

    pub fn move_by_velocity(&mut self, delta_time: f32) {
        self.position = runge_kutta(self.position, delta_time, self.velocity);
    }
    
    pub fn predict_position(&mut self, delta_time: f32) {
        self.predicted_position = runge_kutta(self.position, delta_time, self.velocity);
    }

}
