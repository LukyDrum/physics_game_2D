use crate::game::draw_slider;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
};

use super::{ColorPicker, SLIDER_HEIGHT};

/// Minimum density for fluids - this is somewhere between the density of Hydrogen and Helium.
const MIN_DENSITY: f32 = 0.1;
/// Maximum density for fluids - this is the density of Mercury at room temeprature.
const MAX_DENSITY: f32 = 13.5;
/// Default density - water
const DEFAULT_DENSITY: f32 = 1.0;

pub struct FluidSelector {
    density: f32,
    color_picker: ColorPicker,
}

impl Default for FluidSelector {
    fn default() -> Self {
        FluidSelector {
            density: DEFAULT_DENSITY,
            color_picker: ColorPicker::new(Color::rgb(10, 24, 189)),
        }
    }
}

impl UIComponent for FluidSelector {
    fn draw(&mut self, offset: Vector2<f32>) {
        self.draw_density_selector(offset);
        self.color_picker
            .draw(offset + v2!(0.0, SLIDER_HEIGHT + 10.0));
    }
}

impl FluidSelector {
    pub fn density(&self) -> f32 {
        self.density
    }

    pub fn color(&self) -> Color {
        self.color_picker.color()
    }

    fn draw_density_selector(&mut self, offset: Vector2<f32>) {
        draw_slider(
            offset,
            "Density [g/cm^3]",
            350.0,
            &mut self.density,
            MIN_DENSITY..MAX_DENSITY,
        );
    }
}
