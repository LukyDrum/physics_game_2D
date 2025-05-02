use std::collections::LinkedList;
use std::fs::{read_dir, File};
use std::io::prelude::*;
use std::path::Path;

use crate::serialization::GameSerializedForm;

static ROOT: &'static str = "./";

pub fn save(game_ser_form: GameSerializedForm, name: String) {
    let json = serde_json::to_string_pretty(&game_ser_form)
        .expect("Save failed: failed to serialize to JSON.");

    let path = Path::new(ROOT).join(format!("saves/{name}"));

    let mut file = File::create(path).unwrap();
    file.write_all(&json.into_bytes())
        .expect("Save failed: failed to write data to file.");
}

pub fn list_saves() -> LinkedList<String> {
    let path = Path::new(ROOT).join("saves/");
    let paths = read_dir(path).expect("Failed to read directory.");

    paths
        .map(|p| p.unwrap().file_name().to_str().unwrap().to_owned())
        .inspect(|p| println!("{p:?}"))
        .collect()
}
