mod game;
mod math;
mod physics;
mod rendering;
mod serialization;
mod shapes;
mod utility;

use game::Game;
use macroquad::prelude::*;

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
    let mut game = Game::new(500, 500);

    loop {
        game.update();

        next_frame().await
    }
}
