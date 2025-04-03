use crate::math::Vector2;

use super::{FluidSelector, UIComponent};

/// The UI used to control the game while playing.
/// Allows to control simulation parameters, create things, and more.
pub struct InGameUI {
    fluid_selector: FluidSelector,
}

impl InGameUI {
    pub fn new() -> Self {
        InGameUI {
            fluid_selector: FluidSelector {},
        }
    }
}

impl UIComponent for InGameUI {
    fn draw(&self, offset: Vector2<f32>) {
        self.fluid_selector.draw(offset);
    }
}
