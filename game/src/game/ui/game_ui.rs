use macroquad::{
    shapes::draw_rectangle,
    text::draw_text,
    ui::{root_ui, widgets::Button},
};

use crate::{
    game::config::*,
    math::{v2, Vector2},
    rendering::Color,
    utility::AsMq,
};

use super::{BodyMaker, FluidSelector, InfoPanel, SavesLoads, UIComponent, UIEdit};

pub const FONT_SIZE_LARGE: f32 = 36.0;
pub const FONT_SIZE_MEDIUM: f32 = 26.0;
pub const FONT_SIZE_SMALL: f32 = 16.0;

const TOOL_BUTTON_WIDTH: f32 = 110.0;
const TOOL_BUTTON_HEIGHT: f32 = 25.0;
const TOOL_BUTTON_GAP: f32 = 40.0;
const TOOL_BUTTON_SELECTED_OUTLINE: f32 = 4.0;

#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Info,
    Fluid,
    Rigidbody,
    Configuration,
    SaveLoads,
}

/// The UI used to control the game while playing.
/// Allows to control simulation parameters, create things, and more.
pub struct InGameUI {
    pub fluid_selector: FluidSelector,
    pub info_panel: InfoPanel,
    pub save_loads: SavesLoads,
    pub body_maker: BodyMaker,

    pub selected_tool: Tool,
}

impl Default for InGameUI {
    fn default() -> Self {
        InGameUI {
            fluid_selector: FluidSelector::default(),
            info_panel: InfoPanel::default(),
            save_loads: SavesLoads::default(),
            body_maker: BodyMaker::default(),

            selected_tool: Tool::Info,
        }
    }
}

impl InGameUI {
    pub fn draw(&mut self, offset: Vector2<f32>, game_config: &mut GameConfig) {
        draw_text(
            "Tools",
            offset.x,
            offset.y,
            FONT_SIZE_LARGE,
            Color::rgb(0, 0, 0).as_mq(),
        );
        let offset = offset + v2!(0.0, 50.0);
        // Scope the inner offsets
        {
            self.draw_tool_button("Info [I]", Tool::Info, offset);

            let offset = offset + v2!(TOOL_BUTTON_WIDTH + TOOL_BUTTON_GAP, 0.0);
            self.draw_tool_button("Fluids [F]", Tool::Fluid, offset);

            let offset = offset + v2!(TOOL_BUTTON_WIDTH + TOOL_BUTTON_GAP, 0.0);
            self.draw_tool_button("Bodies [B]", Tool::Rigidbody, offset);

            let offset = offset + v2!(TOOL_BUTTON_WIDTH + TOOL_BUTTON_GAP, 0.0);
            self.draw_tool_button("Config [C]", Tool::Configuration, offset);

            let offset = offset + v2!(TOOL_BUTTON_WIDTH + TOOL_BUTTON_GAP, 0.0);
            self.draw_tool_button("Saves/Loads [L]", Tool::SaveLoads, offset);
        }

        let offset = offset + v2!(0.0, 50.0);
        match self.selected_tool {
            Tool::Info => self.info_panel.draw(offset),
            Tool::Fluid => self.fluid_selector.draw(offset),
            Tool::Rigidbody => self.body_maker.draw(offset),
            Tool::Configuration => {
                game_config.draw_edit(offset, v2!(80.0, 20.0), "");
            }
            Tool::SaveLoads => self.save_loads.draw(offset),
        };
    }

    fn draw_tool_button(&mut self, title: &'static str, tool: Tool, offset: Vector2<f32>) {
        if self.selected_tool == tool {
            draw_rectangle(
                offset.x - TOOL_BUTTON_SELECTED_OUTLINE,
                offset.y - TOOL_BUTTON_SELECTED_OUTLINE,
                TOOL_BUTTON_WIDTH + TOOL_BUTTON_SELECTED_OUTLINE * 2.0,
                TOOL_BUTTON_HEIGHT + TOOL_BUTTON_SELECTED_OUTLINE * 2.0,
                Color::rgb(0, 0, 0).as_mq(),
            );
        }

        let info_button_clicked = Button::new(title)
            .position(offset.as_mq())
            .size(v2!(TOOL_BUTTON_WIDTH, TOOL_BUTTON_HEIGHT).as_mq())
            .ui(&mut root_ui());
        if info_button_clicked {
            self.selected_tool = tool;
        }
    }
}
