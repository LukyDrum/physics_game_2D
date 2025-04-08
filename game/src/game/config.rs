use game_macros::UIEditable;

use crate::game::UIEdit;
use crate::math::Vector2;

#[derive(Clone)]
pub struct GameConfig {
    pub sph_config: SphConfig,
    pub rb_config: RigidBodiesConfig,
    pub time_step: f32,
    pub sub_steps: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            sph_config: SphConfig::default(),
            rb_config: RigidBodiesConfig::default(),
            time_step: 0.01,
            sub_steps: 2,
        }
    }
}

/// Values for configuring the SPH fluid simulation.
#[derive(Clone, UIEditable)]
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
            base_pressure: 100_000.0,
            base_body_force: 10_000.0,
            gravity: Vector2::new(0.0, 981.0),
        }
    }
}

#[derive(Clone)]
pub struct RigidBodiesConfig {
    pub gravity: Vector2<f32>,
}

impl Default for RigidBodiesConfig {
    fn default() -> Self {
        RigidBodiesConfig {
            gravity: Vector2::new(0.0, 981.0),
        }
    }
}
