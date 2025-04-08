use crate::math::{v2, Vector2};

use super::{FluidSelector, UIComponent, UIEdit};

/// The UI used to control the game while playing.
/// Allows to control simulation parameters, create things, and more.
pub struct InGameUI {
    pub fluid_selector: FluidSelector,
    some_num: f32,
}

impl Default for InGameUI {
    fn default() -> Self {
        InGameUI {
            fluid_selector: FluidSelector::default(),
            some_num: 100.0,
        }
    }
}

impl UIComponent for InGameUI {
    fn draw(&mut self, offset: Vector2<f32>) {
        self.fluid_selector.draw(offset);

        let offset = offset + v2!(0.0, 400.0);
        self.some_num
            .draw_edit(offset, v2!(150.0, 50.0), "Just some number");
    }
}
