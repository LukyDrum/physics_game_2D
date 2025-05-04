use macroquad::text::draw_text;

const TOP_LABEL_GAP: f32 = 15.0;
pub static COLOR_PICKER_HEIGHT: f32 = FONT_SIZE_MEDIUM + TOP_LABEL_GAP + 3.0 * SLIDER_HEIGHT;

use crate::{
    game::{UIComponent, FONT_SIZE_MEDIUM},
    math::{v2, Vector2},
    rendering::Color,
    utility::AsMq,
};

use super::{draw_slider, SLIDER_HEIGHT};

pub struct ColorPicker {
    color: Color,
}

impl ColorPicker {
    pub fn new(default_color: Color) -> Self {
        Self {
            color: default_color,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

impl UIComponent for ColorPicker {
    fn draw(&mut self, offset: Vector2<f32>) {
        let (mut r, mut g, mut b) = (
            self.color.r * 255.0,
            self.color.g * 255.0,
            self.color.b * 255.0,
        );

        draw_text(
            "Color",
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let offset = offset + v2!(0.0, TOP_LABEL_GAP);
        draw_slider(offset, "R", 350.0, &mut r, 0.0..255.0);
        self.color.r = r / 255.0;

        let offset = offset + v2!(0.0, SLIDER_HEIGHT);
        draw_slider(offset, "G", 350.0, &mut g, 0.0..255.0);
        self.color.g = g / 255.0;

        let offset = offset + v2!(0.0, SLIDER_HEIGHT);
        draw_slider(offset, "B", 350.0, &mut b, 0.0..255.0);
        self.color.b = b / 255.0;
    }
}
