mod components;
mod game_ui;

pub use components::*;
pub use game_ui::InGameUI;

use crate::math::Vector2;

pub trait UIComponent {
    /// Draws this component to the screen at the specified offset.
    fn draw(&mut self, offset: Vector2<f32>);
}
