use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
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
