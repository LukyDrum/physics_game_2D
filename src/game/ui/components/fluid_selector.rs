use macroquad::text::draw_text;

use crate::{game::UIComponent, math::Vector2, rendering::Color, utility::AsMq};

pub struct FluidSelector {}

impl UIComponent for FluidSelector {
    fn draw(&self, offset: Vector2<f32>) {
        draw_text(
            "Fluid selector",
            offset.x,
            offset.y,
            35.0,
            Color::rgb(0, 0, 0).as_mq(),
        );
    }
}
