mod simulation;
mod vec2_extension;

use macroquad::prelude::*;
use simulation::{Particle, SimulationConfig};
use vec2_extension::*;

const SIM_CONF: SimulationConfig = SimulationConfig::default();
const RADIUS: f32 = 10.0;

/// Creates the window configruation for Macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: "SPH".to_owned(),
        window_width: 500,
        window_height: 500,
        window_resizable: false,
        ..Default::default()
    }
}

/// Gets the difference in time from last frame.
fn delta_time() -> f32 {
    get_frame_time()
}

/// Checks boundaries and adjusts the particles velocitiy accordingly.
fn resolve_boundaries(particle: &mut Particle) {
    let x = particle.position.x;
    let y = particle.position.y;

    if x < RADIUS || x > screen_width() - RADIUS {
        particle.velocity.flip_x();
        particle.position.x += particle.velocity.x * delta_time();
        particle.velocity.x *= SIM_CONF.collision_damping;
    }
    if y < RADIUS || y > screen_height() - RADIUS {
        particle.velocity.flip_y();
        particle.position.y += particle.velocity.y * delta_time();
        particle.velocity.y *= SIM_CONF.collision_damping;
    }
}

/// Applies gravitational acceleration to the particle.
fn apply_gravity(particle: &mut Particle) {
    particle.velocity += SIM_CONF.gravity * delta_time();
}

/// Moves the particle by it's velocity.
fn move_by_velocity(particle: &mut Particle) {
    particle.position += particle.velocity * delta_time();
}

/// The coordinate system goes from (0, 0) = top-left to (WIDTH, HEIGHT) = bottom-right.
///
///    (0, 0) --------- (WIDTH, 0)
///      |                  |
///      |                  |
///      |                  |
///      |                  |
///  (HEIGHT, 0) --- (WIDTH, HEIGHT)
#[macroquad::main(window_conf)]
async fn main() {
    let mut particle = Particle::new(Vec2::new(250.0, 250.0));

    loop {
        clear_background(GRAY);

        // CORE
        // Simulate
        apply_gravity(&mut particle);
        resolve_boundaries(&mut particle);
        move_by_velocity(&mut particle);

        // Draw
        draw_circle(particle.position.x, particle.position.y, RADIUS, BLUE);

        next_frame().await
    }
}
