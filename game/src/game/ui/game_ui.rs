use crate::{
    game::config::*,
    math::{v2, Vector2},
};

use super::{FluidSelector, UIComponent, UIEdit};

/// The UI used to control the game while playing.
/// Allows to control simulation parameters, create things, and more.
pub struct InGameUI {
    pub fluid_selector: FluidSelector,
    sph_config: SphConfig,
}

impl Default for InGameUI {
    fn default() -> Self {
        InGameUI {
            fluid_selector: FluidSelector::default(),
            sph_config: SphConfig::default(),
        }
    }
}

impl UIComponent for InGameUI {
    fn draw(&mut self, offset: Vector2<f32>) {
        self.fluid_selector.draw(offset);

        let offset = offset + v2!(0.0, 300.0);
        self.sph_config.draw_edit(offset, v2!(100.0, 20.0), "SPH");
    }
}
