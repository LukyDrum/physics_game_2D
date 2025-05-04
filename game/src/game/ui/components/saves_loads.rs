use std::collections::LinkedList;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use chrono::offset;
use macroquad::text::draw_text;
use macroquad::ui::widgets::{Button, InputText};
use macroquad::ui::{root_ui, Skin};

use crate::game::ui::RED_BUTTON_SKIN;
use crate::game::{save_load, FONT_SIZE_MEDIUM};
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
    call_update_next_tick: bool,
}

pub enum SaveLoadAction {
    Nothing,
    Save,
    Load(GameSerializedForm),
}

impl Default for SavesLoads {
    fn default() -> Self {
        let saves = Arc::new(RwLock::new(get_saves()));

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
            call_update_next_tick: false,
        }
    }
}

impl Drop for SavesLoads {
    fn drop(&mut self) {
        self.end_of_checks_flag.store(true, Ordering::Relaxed);
    }
}

fn get_saves() -> LinkedList<String> {
    save_load::list_saves()
        .iter()
        .filter_map(|s| s.strip_suffix(".json").map(|s| s.to_owned()))
        .collect()
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
        update_saves_list(&saves);
    }
}

fn update_saves_list(saves: &Arc<RwLock<LinkedList<String>>>) {
    let new_saves = get_saves();
    let mut write = saves.write().unwrap();
    *write = new_saves;
}

impl UIComponent for SavesLoads {
    fn draw(&mut self, offset: Vector2<f32>) {
        if self.call_update_next_tick {
            self.call_update_next_tick = false;
            update_saves_list(&self.saves);
        }

        if Button::new("Save")
            .size(v2!(80.0, 25.0).as_mq())
            .position(offset.as_mq())
            .ui(&mut root_ui())
        {
            self.action = SaveLoadAction::Save;
            self.call_update_next_tick = true;
            return;
        }

        let offset_input = offset + v2!(120.0, 0.0);
        let old_save_file_name = self.save_file_name.clone();
        InputText::new(42)
            .position(offset_input.as_mq())
            .size(v2!(150.0, 25.0).as_mq())
            .ui(&mut root_ui(), &mut self.save_file_name);
        // Do not allow names containing '_'
        self.save_file_name = self.save_file_name.replace('_', "");

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
        let og_offset = offset;
        let mut delete_save = None;
        {
            let read = self.saves.read().unwrap();
            for save in &*read {
                let display_name = if save.starts_with('_') {
                    &save[1..]
                } else {
                    save.as_str()
                };

                if Button::new(display_name)
                    .size(v2!(150.0, 25.0).as_mq())
                    .position(offset.as_mq())
                    .ui(&mut root_ui())
                {
                    self.action = SaveLoadAction::Load(save_load::load_save(save));
                    return;
                }

                offset += v2!(0.0, 35.0);
            }

            // Draw a second column of button for deleting
            root_ui().push_skin(RED_BUTTON_SKIN.get().unwrap());
            offset = og_offset;
            for save in &*read {
                let side_offset = offset + v2!(180.0, 0.0);

                // Do not draw delete button for pretected savefiles - containing '_'
                if !save.contains('_') {
                    if Button::new("Delete")
                        .size(v2!(60.0, 25.0).as_mq())
                        .position(side_offset.as_mq())
                        .ui(&mut root_ui())
                    {
                        delete_save = Some(save.clone());
                    }
                }

                offset += v2!(0.0, 35.0);
            }
            root_ui().pop_skin();
        }

        if let Some(save_name) = delete_save {
            save_load::delete_save(save_name.as_str());
            update_saves_list(&self.saves);
        }

        self.action = SaveLoadAction::Nothing;
    }
}
