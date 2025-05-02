use macroquad::ui::root_ui;
use macroquad::ui::widgets::Button;

use crate::serialization::GameSerializedForm;
use crate::utility::AsMq;
use crate::{
    game::UIComponent,
    math::{v2, Vector2},
};

pub struct SavesLoads {
    pub action: SaveLoadAction,
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
        } else {
            self.action = SaveLoadAction::Nothing;
        }
    }
}
