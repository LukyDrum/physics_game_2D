mod game;
mod ui;

pub use game::*;
pub use ui::*;

use crate::{physics::rigidbody::RigidBodiesConfig, SphConfig};

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
