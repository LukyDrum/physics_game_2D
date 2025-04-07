use crate::math::Vector2;

use super::{FluidSelector, UIComponent};

/// The UI used to control the game while playing.
/// Allows to control simulation parameters, create things, and more.
pub struct InGameUI {
    pub fluid_selector: FluidSelector,
}

impl Default for InGameUI {
    fn default() -> Self {
        InGameUI {
            fluid_selector: FluidSelector::default(),
        }
    }
}

impl UIComponent for InGameUI {
    fn draw(&mut self, offset: Vector2<f32>) {
        self.fluid_selector.draw(offset);
    }
}
