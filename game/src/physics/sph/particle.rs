use crate::math::Vector2;
use crate::rendering::Color;
use crate::utility::runge_kutta;

pub(super) const PRESSURE_BASE: f32 = 100_000.0;
pub(super) const BODY_COLLISION_FORCE_BASE: f32 = 10_000.0;

#[derive(Default, Clone)]
pub struct Particle {
    pub position: Vector2<f32>,
    pub predicted_position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub sph_density: f32,
    pub sph_near_density: f32,
    pub(super) mass: f32,
    pub(super) target_density: f32,
    pub(super) pressure_multiplier: f32,
    /// A multiplier of the force on collision with a rigidbody. This is done to simulate a bigger
    /// ammount of fluid hitting the object instead of only a few particles.
    pub(super) body_collision_force: f32,
    pub(super) accumulated_force: Vector2<f32>,
    pub color: Color,
    /// Should be set by the simulation when the particle is inserted
    pub(super) id: u32,
}

impl Particle {
    pub fn new(position: Vector2<f32>) -> Self {
        Self::new_with_velocity(position, Vector2::zero())
    }

    pub fn new_with_velocity(position: Vector2<f32>, velocity: Vector2<f32>) -> Self {
        Particle {
            position,
            predicted_position: position,
            velocity,
            sph_density: 1.0,
            sph_near_density: 1.0,
            mass: 1.0,
            target_density: 1.0,
            pressure_multiplier: PRESSURE_BASE,
            body_collision_force: BODY_COLLISION_FORCE_BASE,
            accumulated_force: Vector2::zero(),
            color: Color::rgb(0, 0, 255),
            id: 0,
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.set_mass(mass);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn set_mass(&mut self, new_mass: f32) {
        self.mass = new_mass;
        self.target_density = new_mass;
        self.pressure_multiplier = PRESSURE_BASE / self.mass;
        self.body_collision_force = BODY_COLLISION_FORCE_BASE * self.mass;
    }

    /// Adds `force` to the accumulated force.
    pub fn add_force(&mut self, force: Vector2<f32>) {
        self.accumulated_force += force;
    }

    pub fn apply_accumulated_force(&mut self, delta_time: f32) {
        if self.accumulated_force.length_squared() < 0.001 {
            return;
        }

        let acceleration = self.accumulated_force / self.mass;

        self.velocity = runge_kutta(self.velocity, delta_time, acceleration);
        // Reset the accumulated force
        self.accumulated_force = Vector2::zero();
    }

    pub fn move_by_velocity(&mut self, delta_time: f32) {
        self.position = runge_kutta(self.position, delta_time, self.velocity);
    }

    pub fn predict_position(&mut self, delta_time: f32) {
        self.predicted_position = runge_kutta(self.position, delta_time, self.velocity);
    }

    pub fn pressure(&self) -> f32 {
        self.pressure_multiplier * (self.sph_density - self.target_density)
    }

    pub fn near_pressure(&self) -> f32 {
        self.sph_near_density
    }
}
