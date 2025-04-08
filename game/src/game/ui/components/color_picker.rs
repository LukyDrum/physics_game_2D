use macroquad::ui::{root_ui, widgets::InputText};

use crate::{game::UIComponent, math::Vector2, rendering::Color, utility::AsMq};

const INPUT_WIDTH: f32 = 100.0;
const SPACING: f32 = 120.0;

pub struct ColorPicker {
    color: Color,
    r_str: String,
    g_str: String,
    b_str: String,
}

impl ColorPicker {
    pub fn new(default_color: Color) -> Self {
        Self {
            color: default_color,
            r_str: String::new(),
            g_str: String::new(),
            b_str: String::new(),
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

impl UIComponent for ColorPicker {
    fn draw(&mut self, offset: Vector2<f32>) {
        InputText::new(1)
            .label("R")
            .position(offset.as_mq())
            .size(Vector2::new(INPUT_WIDTH, 20.0).as_mq())
            .filter_numbers()
            .ui(&mut root_ui(), &mut self.r_str);
        InputText::new(2)
            .label("G")
            .position((offset + Vector2::new(SPACING, 0.0)).as_mq())
            .size(Vector2::new(INPUT_WIDTH, 20.0).as_mq())
            .filter_numbers()
            .ui(&mut root_ui(), &mut self.g_str);
        InputText::new(3)
            .label("B")
            .position((offset + Vector2::new(2.0 * SPACING, 0.0)).as_mq())
            .size(Vector2::new(INPUT_WIDTH, 20.0).as_mq())
            .filter_numbers()
            .ui(&mut root_ui(), &mut self.b_str);

        for (str, color_channel) in [
            (&mut self.r_str, &mut self.color.r),
            (&mut self.g_str, &mut self.color.g),
            (&mut self.b_str, &mut self.color.b),
        ] {
            if let Ok(value) = str.parse::<u8>() {
                *color_channel = value as f32 / 255.0;
            } else {
                *str = ((*color_channel * 255.0) as u8).to_string();
            }
        }
    }
}
