use std::fs::File;
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
