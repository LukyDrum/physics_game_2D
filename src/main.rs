mod lookup;
mod simulation;
mod vec2_extension;
mod linked_linked_list;

use lookup::LookUp;
use macroquad::prelude::*;
use simulation::{Particle, SimulationConfig};
use vec2_extension::*;

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
    1.0 / 24.0
}

fn make_grid_of_particles(count: usize, top_left: Vec2, spacing: f32) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(count);
    let root = (count as f32).sqrt() as usize;
    for x in 0..root {
        let x = top_left.x + x as f32 * spacing;
        for y in 0..root {
            let y = top_left.y + y as f32 * spacing;
            particles.push(Particle::new(Vec2::new(x, y)));
        }
    }

    particles
}

fn kernel(dist: f32) -> f32 {
    if dist > SIM_CONF.smoothing_radius {
        return 0.0;
    }

    (1.0 - dist / SIM_CONF.smoothing_radius).max(0.0).powi(2)
}

fn kernel_derivative(dist: f32) -> f32 {
    if dist > SIM_CONF.smoothing_radius {
        return 0.0;
    }

    (2.0 * (dist - SIM_CONF.smoothing_radius)) / (SIM_CONF.smoothing_radius.powi(2))
}

fn density_to_pressure(density: f32) -> f32 {
    SIM_CONF.pressure_multiplier * (density - SIM_CONF.target_density)
}

/// Checks boundaries and adjusts the particles velocitiy accordingly.
fn resolve_boundaries(particle: &mut Particle) {
    let x = particle.position.x;
    let y = particle.position.y;

    let mut flip_x = false;
    let mut flip_y = false;

    if x < RADIUS || x > screen_width() - RADIUS {
        flip_x = true;
    }
    if y < RADIUS || y > screen_height() - RADIUS {
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
        particle.position.x += particle.velocity.x * delta_time();
        particle.velocity.x *= SIM_CONF.collision_damping;
    }
    if flip_y {
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

fn predict_position(particle: &mut Particle) {
    particle.predicted_position = particle.position + particle.velocity * delta_time();
}

fn calculate_densities(particles: &mut Vec<Particle>, lookup: &LookUp) {
    // TODO: Fix this later
    // Lets presume that mass is 1 for all particles
    let mass = 1.0;
    for i in 0..particles.len() {
        let pos = particles[i].predicted_position;
        
        let neighbors = lookup.get_neighbors(pos);
        particles[i].density = neighbors
            .iter()
            .map(|index| mass * kernel((pos - particles[*index].predicted_position).length()))
            .sum();
    }
}

fn apply_pressures(particles: &mut Vec<Particle>, lookup: &LookUp) {
    for i in 0..particles.len() {
        let pos = particles[i].predicted_position;
        let pressure = density_to_pressure(particles[i].density);

        let neighbors = lookup.get_neighbors(pos);
        let pressure_force: Vec2 = neighbors
            .iter()
            .map(|index| {
                let p = particles[*index];
                let pos_diff = p.predicted_position - pos;
                let dir = pos_diff.normalize();
                let other_pressure = density_to_pressure(p.density);
                if dir.is_nan() {
                    Vec2::ZERO
                } else {
                    let shared_pressure = (pressure + other_pressure) / (2.0 * p.density);
                    shared_pressure * kernel_derivative(pos_diff.length()) * dir
                }
            })
            .sum();
        particles[i].velocity += pressure_force * delta_time();
    }
}

fn setup_lookup(lookup: &mut LookUp, particles: &Vec<Particle>) {
    lookup.clear();
    for index in 0..particles.len() {
        lookup.insert(&particles[index], index);
    }
}

fn simulate(particles: &mut Vec<Particle>, lookup: &LookUp) {
    // Predict positions
    particles.iter_mut().for_each(|p| predict_position(p));
    calculate_densities(particles, lookup);
    apply_pressures(particles, lookup);
    particles.iter_mut().for_each(|p| {
        apply_gravity(p);
        move_by_velocity(p);
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
///  (HEIGHT, 0) --- (WIDTH, HEIGHT)
#[macroquad::main(window_conf)]
async fn main() {
    let mut particles = make_grid_of_particles(1750, Vec2::new(5.0, 42.0), 6.0);

    let mut lookup = LookUp::new(WIDTH, HEIGHT, SIM_CONF.smoothing_radius);

    loop {
        clear_background(GRAY);

        // CORE

        setup_lookup(&mut lookup, &particles);
        // Simulate
        simulate(&mut particles, &lookup);
        // Draw
        for p in &particles {
            draw_circle(p.position.x, p.position.y, RADIUS, BLUE);
        }

        // Draw obstacle
        let w = OBSTACLE_BR.x - OBSTACLE_TL.x;
        let h = OBSTACLE_BR.y - OBSTACLE_TL.y;
        draw_rectangle(OBSTACLE_TL.x, OBSTACLE_TL.y, w, h, YELLOW);

        println!("FPS: {}", get_fps());
        next_frame().await
    }
}
