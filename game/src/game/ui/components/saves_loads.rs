use std::collections::LinkedList;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

const RECHECK_TIME: u64 = 3;

pub struct SavesLoads {
    pub action: SaveLoadAction,
    saves: Arc<RwLock<LinkedList<String>>>,
    _check_handle: JoinHandle<()>,
    end_of_checks_flag: Arc<AtomicBool>,
    pub save_file_name: String,
    pub taken_input: bool,
}

pub enum SaveLoadAction {
    Nothing,
    Save,
    Load(GameSerializedForm),
}

impl Default for SavesLoads {
    fn default() -> Self {
        let saves = Arc::new(RwLock::new(
            save_load::list_saves()
                .iter()
                .filter_map(|s| s.strip_suffix(".json").map(|s| s.to_owned()))
                .collect(),
        ));

        let end_of_checks = Arc::new(AtomicBool::new(false));
        let handle = {
            let saves = saves.clone();
            let end_of_checks = end_of_checks.clone();
            thread::spawn(move || {
                periodicly_check_save_files(saves, end_of_checks);
            })
        };

        SavesLoads {
            action: SaveLoadAction::Nothing,
            saves,
            _check_handle: handle,
            end_of_checks_flag: end_of_checks,
            save_file_name: "save-1".to_owned(),
            taken_input: false,
        }
    }
}

impl Drop for SavesLoads {
    fn drop(&mut self) {
        self.end_of_checks_flag.store(true, Ordering::Relaxed);
    }
}

fn periodicly_check_save_files(
    saves: Arc<RwLock<LinkedList<String>>>,
    end_of_checks: Arc<AtomicBool>,
) -> () {
    loop {
        if end_of_checks.load(Ordering::Relaxed) {
            return;
        }

        thread::sleep(Duration::from_secs(RECHECK_TIME));

        let new_saves = save_load::list_saves();
        let mut write = saves.write().unwrap();
        *write = new_saves;
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
        let old_save_file_name = self.save_file_name.clone();
        InputText::new(42)
            .position(offset_input.as_mq())
            .size(v2!(150.0, 25.0).as_mq())
            .ui(&mut root_ui(), &mut self.save_file_name);

        // Compare old and new
        self.taken_input = self.save_file_name != old_save_file_name;

        let mut offset = offset + v2!(0.0, 80.0);
        draw_text(
            "Save files:",
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        offset += v2!(0.0, 50.0);
        let read = self.saves.read().unwrap();
        for save in &*read {
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
