use std::collections::LinkedList;
use std::fs::{self, read_dir, File};
use std::io::prelude::*;
use std::path::Path;

use crate::serialization::GameSerializedForm;

static ROOT: &'static str = "./";

pub fn save(game_ser_form: GameSerializedForm, name: &str) {
    let json = serde_json::to_string_pretty(&game_ser_form)
        .expect("Save failed: failed to serialize to JSON.");

    let full_name = if name.ends_with(".json") {
        name.to_owned()
    } else {
        format!("{name}.json")
    };
    let path = Path::new(ROOT).join(format!("saves/{full_name}"));

    let mut file = File::create(path).unwrap();
    file.write_all(&json.into_bytes())
        .expect("Save failed: failed to write data to file.");
}

pub fn list_saves() -> LinkedList<String> {
    let path = Path::new(ROOT).join("saves/");
    let paths = read_dir(path).expect("Failed to read directory.");

    paths
        .map(|p| p.unwrap().file_name().to_str().unwrap().to_owned())
        .collect()
}

pub fn load_save(save_name: &str) -> GameSerializedForm {
    let path = Path::new(ROOT).join(format!("saves/{save_name}.json"));

    let mut file = File::open(path).expect("Load failed: failed to open file.");

    let mut json = String::new();
    let _ = file.read_to_string(&mut json);

    serde_json::from_str(json.as_str()).expect("Load failed: failed to deserialize from JSON.")
}

pub fn delete_save(save_name: &str) {
    let path = Path::new(ROOT).join(format!("saves/{save_name}.json"));
    let _ = fs::remove_file(path);
}
