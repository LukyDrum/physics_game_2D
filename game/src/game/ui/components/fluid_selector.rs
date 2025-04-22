use macroquad::ui::{root_ui, widgets::Slider};

use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
    utility::AsMq,
};

use super::ColorPicker;

/// Minimum density for fluids - this is somewhere between the density of Hydrogen and Helium.
const MIN_DENSITY: f32 = 0.1;
/// Maximum density for fluids - this is the density of Mercury at room temeprature.
const MAX_DENSITY: f32 = 13.5;
/// Default density - water
const DEFAULT_DENSITY: f32 = 1.0;

const SLIDER_HEIGHT: f32 = 20.0;

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
        Slider::new(0, MIN_DENSITY..MAX_DENSITY)
            .label("Density [g/cm^3]")
            .position(offset.as_mq())
            .size(v2!(350.0, SLIDER_HEIGHT).as_mq())
            .ui(&mut root_ui(), &mut self.density);
        // Clamp the density into the range - in case of change using the input box
        self.density = self.density.clamp(MIN_DENSITY, MAX_DENSITY);
    }
}
