use crate::game::draw_slider;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
};

use super::{ColorPicker, SLIDER_HEIGHT};

const MIN_SIZE: f32 = 5.0;
const DEFAULT_MAX_SIZE: f32 = 500.0;
const MIN_MASS: f32 = 500.0;
const MAX_MASS: f32 = 50_000.0;
const MIN_ORIENTATION: f32 = 0.0;
const MAX_ORIENTATION: f32 = 360.0;

pub struct BodyMaker {
    width: f32,
    height: f32,
    mass: f32,
    orientation: f32,

    max_size: f32,

    color_picker: ColorPicker,
}

impl Default for BodyMaker {
    fn default() -> Self {
        BodyMaker {
            width: 30.0,
            height: 30.0,
            mass: 5000.0,
            orientation: 0.0,

            max_size: DEFAULT_MAX_SIZE,

            color_picker: ColorPicker::new(Color::rgb(0, 0, 0)),
        }
    }
}

impl UIComponent for BodyMaker {
    fn draw(&mut self, offset: Vector2<f32>) {
        draw_slider(
            offset,
            "Width [cm]",
            370.0,
            &mut self.width,
            MIN_SIZE..self.max_size,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + 10.0);
        draw_slider(
            offset,
            "Height [cm]",
            360.0,
            &mut self.height,
            MIN_SIZE..self.max_size,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + 10.0);
        draw_slider(
            offset,
            "Orientation [degrees]",
            360.0,
            &mut self.orientation,
            MIN_ORIENTATION..MAX_ORIENTATION,
        );

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + 10.0);
        draw_slider(
            offset,
            "Mass [g]",
            360.0,
            &mut self.mass,
            MIN_MASS..MAX_MASS,
        );

        self.color_picker
            .draw(offset + v2!(0.0, SLIDER_HEIGHT + 25.0));
    }
}

impl BodyMaker {
    pub fn color(&self) -> Color {
        self.color_picker.color()
    }

    pub fn size(&self) -> Vector2<f32> {
        v2!(self.width, self.height)
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn orientation(&self) -> f32 {
        self.orientation
    }

    pub fn set_max_size(&mut self, new_max: f32) {
        self.max_size = new_max;
    }
}
