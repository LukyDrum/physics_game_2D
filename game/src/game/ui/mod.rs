mod components;
mod game_ui;

pub use components::*;
pub use game_ui::*;
use macroquad::ui::{
    root_ui,
    widgets::{InputText, Label},
};

use crate::{
    math::{v2, Vector2},
    utility::AsMq,
};

pub trait UIComponent {
    /// Draws this component to the screen at the specified offset.
    fn draw(&mut self, offset: Vector2<f32>);
}

/// Draws this type to the screen just as `UIComponent` but specificly for edititng.
/// `input_size` is the size of the individual input boxes.
/// Returns the total size it occupies.
pub trait UIEdit {
    fn draw_edit(
        &mut self,
        position: Vector2<f32>,
        input_size: Vector2<f32>,
        label: &str,
    ) -> Vector2<f32>;
}

fn id_from_position(position: Vector2<f32>) -> u64 {
    position.x as u64 * 47951 + position.y as u64 * 34807
}

macro_rules! ui_edit_numbers {
    ($type:ty) => {
        impl UIEdit for $type {
            fn draw_edit(
                &mut self,
                position: Vector2<f32>,
                size: Vector2<f32>,
                label: &str,
            ) -> Vector2<f32> {
                let mut input = self.to_string();
                InputText::new(id_from_position(position))
                    .filter_numbers()
                    .position(position.as_mq())
                    .size(size.as_mq())
                    .label(label)
                    .label_font_size(FONT_SIZE_SMALL)
                    .input_font_size(FONT_SIZE_SMALL)
                    .ui(&mut root_ui(), &mut input);

                if let Ok(parsed) = input.parse::<$type>() {
                    *self = parsed;
                }

                size
            }
        }
    };
}

ui_edit_numbers!(u8);
ui_edit_numbers!(u32);
ui_edit_numbers!(i32);
ui_edit_numbers!(f32);

impl UIEdit for Vector2<f32> {
    fn draw_edit(
        &mut self,
        position: Vector2<f32>,
        size: Vector2<f32>,
        label: &str,
    ) -> Vector2<f32> {
        Label::new(label)
            .position(position.as_mq())
            .ui(&mut root_ui());

        let position = position + v2!(0.0, size.y);
        self.x.draw_edit(position, size, "X");
        self.y
            .draw_edit(position + v2!(size.x * 1.4, 0.0), size, "Y");

        // Y * C for the label above
        v2!(size.x * 2.2, size.y * 1.1)
    }
}
