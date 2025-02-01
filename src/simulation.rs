use macroquad::math::Vec2;

pub struct SimulationConfig {
    pub gravity: Vec2,
    pub collision_damping: f32,
}

impl SimulationConfig {
    /// NOT implementation of Default trait, but a custom `const` function
    pub const fn default() -> Self {
        SimulationConfig { gravity: Vec2::new(0.0, 9.81), collision_damping: 0.7 }
    }
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self::new_with_velocity(position, Vec2::ZERO)
    }

    pub fn new_with_velocity(position: Vec2, velocity: Vec2) -> Self {
        Particle { position, velocity }
    }
}
