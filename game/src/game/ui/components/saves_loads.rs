use std::collections::LinkedList;

use macroquad::ui::root_ui;
use macroquad::ui::widgets::Button;

use crate::game::save_load;
use crate::serialization::GameSerializedForm;
use crate::utility::AsMq;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
};

pub struct SavesLoads {
    pub action: SaveLoadAction,
    saves: LinkedList<String>,
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

        let mut offset = offset + v2!(150.0, 0.0);
        for save in &self.saves {
            if Button::new(save.as_str())
                .size(v2!(100.0, 25.0).as_mq())
                .position(offset.as_mq())
                .ui(&mut root_ui())
            {
                println!("Selected {save}");
                return;
            }

            offset += v2!(0.0, 35.0);
        }

        self.action = SaveLoadAction::Nothing;
    }
}
