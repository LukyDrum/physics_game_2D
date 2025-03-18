mod math;
mod physics;
mod rendering;
// mod speed_test;
mod game;
mod utility;

use game::Game;
use macroquad::prelude::*;

use crate::physics::sph::*;

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 500.0;

/// Creates the window configruation for Macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: "Physics Game".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: true,
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
    let mut game = Game::new(WIDTH as usize, HEIGHT as usize);

    loop {
        game.handle_input();
        game.update();
        game.draw();

        println!("FPS: {}", get_fps());
        next_frame().await
    }
}
