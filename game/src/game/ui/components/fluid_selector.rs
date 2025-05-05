use macroquad::text::draw_text;
use macroquad::ui::root_ui;
use macroquad::ui::widgets::Button;

use crate::game::ui::RED_BUTTON_SKIN;
use crate::game::{draw_slider, FONT_SIZE_SMALL};
use crate::utility::AsMq;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
};

use super::{ColorPicker, GAP, SLIDER_HEIGHT, SLIDER_LENGTH};

/// Minimum density for fluids - this is somewhere between the density of Hydrogen and Helium.
const MIN_DENSITY: f32 = 0.1;
/// Maximum density for fluids - this is the density of Mercury at room temeprature.
const MAX_DENSITY: f32 = 13.5;
/// Default density - water
const DEFAULT_DENSITY: f32 = 1.0;

const TUTORIAL_LINES: [&str; 1] = ["[Left MB] - Spawn fluid"];

#[derive(Clone, Copy)]
pub enum FluidSelectorAction {
    Nothing,
    ClearParticles,
}

pub struct FluidSelector {
    pub density: f32,
    color_picker: ColorPicker,
    pub action: FluidSelectorAction,
    pub droplet_count: u32,
}

impl Default for FluidSelector {
    fn default() -> Self {
        FluidSelector {
            density: DEFAULT_DENSITY,
            color_picker: ColorPicker::new(Color::rgb(10, 24, 189)),
            action: FluidSelectorAction::Nothing,
            droplet_count: 4,
        }
    }
}

impl UIComponent for FluidSelector {
    fn draw(&mut self, offset: Vector2<f32>) {
        let mut offset = offset;
        for line in TUTORIAL_LINES {
            draw_text(
                line,
                offset.x,
                offset.y,
                FONT_SIZE_SMALL,
                Color::rgb(0, 0, 0).as_mq(),
            );
            offset += v2!(0.0, FONT_SIZE_SMALL + 10.0);
        }

        root_ui().push_skin(RED_BUTTON_SKIN.get().unwrap());
        if Button::new("Clear fluid")
            .size(v2!(100.0, 25.0).as_mq())
            .position(offset.as_mq())
            .ui(&mut root_ui())
        {
            self.action = FluidSelectorAction::ClearParticles;
        } else {
            self.action = FluidSelectorAction::Nothing;
        }
        root_ui().pop_skin();

        let offset = offset + v2!(0.0, 45.0);
        self.draw_density_selector(offset);

        let offset = offset + v2!(0.0, SLIDER_HEIGHT + GAP);
        let mut f_count = self.droplet_count as f32;
        draw_slider(
            offset,
            "Droplet count",
            SLIDER_LENGTH,
            &mut f_count,
            1.0..10.0,
        );
        self.droplet_count = f_count.round() as u32;

        self.color_picker
            .draw(offset + v2!(0.0, SLIDER_HEIGHT + 25.0));
    }
}

impl FluidSelector {
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
