mod math;
mod physics;
mod rendering;
mod speed_test;
mod utility;

use macroquad::prelude::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rendering::{Renderer, ScalarFieldRenderer};

use crate::math::Vector2;
use crate::physics::sph::*;
use crate::utility::runge_kutta;

const WIDTH: f32 = 500.0;
const HEIGHT: f32 = 500.0;

const SIM_CONF: SimulationConfig = SimulationConfig::default();
const RADIUS: f32 = 4.5;

const OBSTACLE_TL: Vec2 = Vec2::new(350.0, 200.0);
const OBSTACLE_BR: Vec2 = Vec2::new(450.0, 450.0);

/// Creates the window configruation for Macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: "SPH".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

/// Gets the difference in time from last frame.
fn delta_time() -> f32 {
    // get_frame_time()
    1.0 / 15.0
}

/// Checks boundaries and adjusts the particles velocitiy accordingly.
fn resolve_boundaries(particle: &mut Particle) {
    // TODO: Rewrite boundary checks - this is horrible
    let x = particle.position.x;
    let y = particle.position.y;

    let mut flip_x = false;
    let mut flip_y = false;

    if !(RADIUS..=WIDTH - RADIUS).contains(&x) {
        flip_x = true;
    }
    if !(RADIUS..=HEIGHT - RADIUS).contains(&y) {
        flip_y = true;
    }

    // Hacky check for the obstacle
    if x + RADIUS > OBSTACLE_TL.x
        && x - RADIUS < OBSTACLE_BR.x
        && y + RADIUS > OBSTACLE_TL.y
        && y - RADIUS < OBSTACLE_BR.y
    {
        if x + RADIUS > OBSTACLE_TL.x && x - RADIUS < OBSTACLE_BR.x {
            flip_x = true;
        }
        if y + RADIUS > OBSTACLE_TL.y && y - RADIUS < OBSTACLE_BR.y {
            flip_y = true;
        }
    }

    if flip_x {
        particle.velocity.flip_x();
        particle.position.x = runge_kutta(particle.position.x, delta_time(), particle.velocity.x);
        particle.velocity.x *= SIM_CONF.collision_damping;
    }
    if flip_y {
        particle.velocity.flip_y();
        particle.position.y = runge_kutta(particle.position.y, delta_time(), particle.velocity.y);
        particle.velocity.y *= SIM_CONF.collision_damping;
    }
}

/// The "heavy" part of the game (simulation and such) that does not require user interaction.
/// Given the state of the individual systems it can progress.
/// Is `pub` for use in speed tests.
pub fn simulation_core(sph: &mut Sph) {
    sph.step(delta_time());
    sph.particles.par_iter_mut().for_each(|p| {
        resolve_boundaries(p);
    });
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
    let mut sph = Sph::new(SIM_CONF, WIDTH, HEIGHT);

    let mut renderer: Box<dyn Renderer> =
        Box::new(ScalarFieldRenderer::new(WIDTH as usize, HEIGHT as usize, 5.0).unwrap());

    loop {
        clear_background(GRAY);

        // INPUT
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let new_particle = Particle::new(Vector2::new(mouse_pos.0, mouse_pos.1));
            sph.add_particle(new_particle);
        }

        // CORE
        simulation_core(&mut sph);

        // Draw
        renderer.setup(&sph);
        renderer.draw();

        /*
        for p in &sph.particles {
            let color = if p.mass() < 0.2 {
                WHITE
            } else if p.mass() < 1.0 {
                BLUE
            } else {
                RED
            };
            draw_circle(p.position.x, p.position.y, RADIUS, color);
        }
        */

        // Draw obstacle
        let w = OBSTACLE_BR.x - OBSTACLE_TL.x;
        let h = OBSTACLE_BR.y - OBSTACLE_TL.y;
        draw_rectangle(OBSTACLE_TL.x, OBSTACLE_TL.y, w, h, YELLOW);

        println!("FPS: {}", get_fps());
        next_frame().await
    }
}
