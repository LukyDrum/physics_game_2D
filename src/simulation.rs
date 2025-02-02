use macroquad::math::Vec2;

pub struct SimulationConfig {
    pub gravity: Vec2,
    pub collision_damping: f32,
    pub smoothing_radius: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function
    pub const fn default() -> Self {
        SimulationConfig { gravity: Vec2::new(0.0, 9.81), collision_damping: 0.7, smoothing_radius: 10.0 }
    }
}

#[derive(Default)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub density: f32,
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self::new_with_velocity(position, Vec2::ZERO)
    }

    pub fn new_with_velocity(position: Vec2, velocity: Vec2) -> Self {
        Particle { position, velocity, density: 1.0 }
    }
}
