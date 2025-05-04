mod body_maker;
mod color_picker;
mod fluid_selector;
mod info;
mod saves_loads;

use std::ops::Range;

pub use body_maker::BodyMaker;
pub use color_picker::ColorPicker;
pub use fluid_selector::{FluidSelector, FluidSelectorAction};
pub use info::{EntityInfo, InfoPanel};
pub use saves_loads::{SaveLoadAction, SavesLoads};

use macroquad::ui::{root_ui, widgets::Slider};

use crate::{
    math::{v2, Vector2},
    utility::AsMq,
};

use super::id_from_position;

const SLIDER_HEIGHT: f32 = 20.0;
const SLIDER_LENGTH: f32 = 360.0;
const GAP: f32 = 10.0;

pub fn draw_slider(
    offset: Vector2<f32>,
    label: &'static str,
    length: f32,
    value: &mut f32,
    range: Range<f32>,
) {
    Slider::new(id_from_position(offset), range.clone())
        .label(label)
        .position(offset.as_mq())
        .size(v2!(length, SLIDER_HEIGHT).as_mq())
        .ui(&mut root_ui(), value);
    // Clamp the value into the range - in case of change using the input box
    *value = value.clamp(range.start, range.end);
}
