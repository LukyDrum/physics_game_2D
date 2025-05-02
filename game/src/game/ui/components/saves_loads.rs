use std::collections::LinkedList;

use macroquad::text::draw_text;
use macroquad::ui::root_ui;
use macroquad::ui::widgets::{Button, InputText, Label};

use crate::game::{save_load, FONT_SIZE_LARGE, FONT_SIZE_MEDIUM};
use crate::rendering::Color;
use crate::serialization::GameSerializedForm;
use crate::utility::AsMq;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
};

pub struct SavesLoads {
    pub action: SaveLoadAction,
    saves: LinkedList<String>,
    pub save_file_name: String,
}

pub enum SaveLoadAction {
    Nothing,
    Save,
    Load(GameSerializedForm),
}

impl Default for SavesLoads {
    fn default() -> Self {
        SavesLoads {
            action: SaveLoadAction::Nothing,
            saves: save_load::list_saves()
                .iter()
                .filter_map(|s| s.strip_suffix(".json").map(|s| s.to_owned()))
                .collect(),
            save_file_name: "save-1".to_owned(),
        }
    }
}

impl UIComponent for SavesLoads {
    fn draw(&mut self, offset: Vector2<f32>) {
        if Button::new("Save")
            .size(v2!(80.0, 25.0).as_mq())
            .position(offset.as_mq())
            .ui(&mut root_ui())
        {
            self.action = SaveLoadAction::Save;
            return;
        }

        let offset_input = offset + v2!(120.0, 0.0);
        InputText::new(42)
            .position(offset_input.as_mq())
            .size(v2!(150.0, 25.0).as_mq())
            .ui(&mut root_ui(), &mut self.save_file_name);

        let mut offset = offset + v2!(0.0, 80.0);
        draw_text(
            "Save files:",
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        offset += v2!(0.0, 50.0);
        for save in &self.saves {
            if Button::new(save.as_str())
                .size(v2!(100.0, 25.0).as_mq())
                .position(offset.as_mq())
                .ui(&mut root_ui())
            {
                self.action = SaveLoadAction::Load(save_load::load_save(save));
                return;
            }

            offset += v2!(0.0, 35.0);
        }

        self.action = SaveLoadAction::Nothing;
    }
}
