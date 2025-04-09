use game_macros::UIEditable;

use crate::game::{ui::FONT_SIZE_MEDIUM, UIEdit};
use crate::math::Vector2;
use crate::rendering::Color;
use crate::utility::AsMq;

use macroquad::text::draw_text;

#[derive(Clone, UIEditable)]
pub struct GameConfig {
    #[display_as("Time Step [s]")]
    pub time_step: f32,
    /// This will divide the `time_step` into **n** parts and perform **n** steps of the physical simulation
    /// with those time steps. Leads to better accuracy at cost of performance.
    pub sub_steps: u8,
    #[display_as("Fluids")]
    pub sph_config: SphConfig,
    #[display_as("Rigidbodies")]
    pub rb_config: RigidBodiesConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            time_step: 0.01,
            sub_steps: 2,
            sph_config: SphConfig::default(),
            rb_config: RigidBodiesConfig::default(),
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
    #[display_as("Gravity [cm/s]")]
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

#[derive(Clone, UIEditable)]
pub struct RigidBodiesConfig {
    #[display_as("Gravity [cm/s]")]
    pub gravity: Vector2<f32>,
}

impl Default for RigidBodiesConfig {
    fn default() -> Self {
        RigidBodiesConfig {
            gravity: Vector2::new(0.0, 981.0),
        }
    }
}
