use macroquad::text::draw_text;

use crate::{
    game::config::*,
    math::{v2, Vector2},
    rendering::Color,
    utility::AsMq,
};

use super::{FluidSelector, UIComponent, UIEdit};

pub const FONT_SIZE_LARGE: f32 = 35.0;
pub const FONT_SIZE_MEDIUM: f32 = 25.0;
pub const FONT_SIZE_SMALL: f32 = 15.0;

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

impl InGameUI {
    pub fn draw(&mut self, offset: Vector2<f32>, game_config: &mut GameConfig) {
        self.fluid_selector.draw(offset);
        let offset = offset + v2!(0.0, 200.0);
        draw_text(
            "Configuration",
            offset.x,
            offset.y,
            FONT_SIZE_LARGE,
            Color::rgb(0, 0, 0).as_mq(),
        );
        game_config.draw_edit(offset + v2!(0.0, FONT_SIZE_LARGE), v2!(80.0, 20.0), "");
    }
}
