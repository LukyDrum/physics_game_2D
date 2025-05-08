mod game;
mod math;
mod physics;
mod rendering;
mod serialization;
mod shapes;
mod utility;

use game::Game;
use macroquad::{prelude::*, ui::root_ui};
use rendering::Color;
use utility::AsMq;

use crate::physics::sph::*;

const WIDTH: f32 = 1000.0;
const HEIGHT: f32 = 800.0;

/// Creates the window configruation for Macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: "Physics Game".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: true,
        fullscreen: true,
        ..Default::default()
    }
}

/// The coordinate system goes from (0, 0) = top-left to (WIDTH, HEIGHT) = bottom-right.
///
///    (0, 0) --------- (WIDTH, 0)
///      |                  |
///      |                  |
///      |                  |
///      |                  |
///  (0, HEIGHT) --- (WIDTH, HEIGHT)
#[macroquad::main(window_conf)]
async fn main() {
    // Setup styling
    let checkbox_color_selected = Color::rgb(10, 240, 10).as_mq();
    let checkbox_style = root_ui()
        .style_builder()
        .color_selected(checkbox_color_selected)
        .color_selected_hovered(checkbox_color_selected)
        .build();
    let mut skin = root_ui().default_skin();
    skin.checkbox_style = checkbox_style;
    root_ui().push_skin(&skin);

    let mut game = Game::new(500, 500);

    while !game.quit_flag {
        game.update();

        next_frame().await
    }
}
