use macroquad::math::Vec2;

pub struct SimulationConfig {
    pub gravity: Vec2,
    pub collision_damping: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function simulating default
    pub const fn default() -> Self {
        SimulationConfig {
            gravity: Vec2::new(0.0, 9.81),
            collision_damping: 0.5,
            smoothing_radius: 15.0,
            target_density: 1.0,
            pressure_multiplier: 300.0,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub predicted_position: Vec2,
    pub velocity: Vec2,
    pub density: f32,
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
            density: 1.0,
        }
    }
}
