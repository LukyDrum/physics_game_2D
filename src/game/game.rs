use macroquad::{
    color::RED, input::{is_key_pressed, is_mouse_button_down, mouse_position, KeyCode, MouseButton}, models::draw_grid, shapes::draw_circle, window::clear_background
};

use crate::{
    math::{v2, Vector2},
    physics::rigidbody::{Body, BoxBody, Line, TriangleBody},
    rendering::{Color, Draw, MarchingSquaresRenderer, Renderer},
    Particle, Sph, WIDTH,
};

pub trait GameBody: Body + Draw + Sync {}
impl GameBody for Line {}
impl GameBody for BoxBody {}
impl GameBody for TriangleBody {}

pub struct Game {
    // Physics stuff
    time_step: f32,
    /// This will divide the `time_step` into **n** parts and perform **n** steps of the physical simulation
    /// with those time steps. Leads to better accuracy at cost of performance.
    step_division: u8,
    fluid_system: Sph,
    /// If the physics are currently being simulated or not
    is_simulating: bool,
    bodies: Vec<Box<dyn GameBody>>,

    // GUI things
    gameview_offset: Vector2<f32>,
    gameview_width: f32,
    gameview_height: f32,
    renderer: Box<dyn Renderer>,
    draw_particles: bool,
}

impl Game {
    /// Creates a new instance of Game with all the system instantiated.
    /// `width` and `height` are the dimensions of the game view / game world.
    pub fn new(width: usize, height: usize) -> Self {
        let (f_width, f_height) = (width as f32, height as f32);

        let sph = Sph::new(f_width, f_height);
        let renderer_step_size = f_width / 100.0;
        // Add basic container
        let bodies: Vec<Box<dyn GameBody>> = vec![
            /*
            Box::new(Container::new(
                v2!(f_width * 0.5, f_height * 0.5),
                f_width,
                f_height,
            )),
            */
            Box::new(BoxBody::new(v2!(10.0, f_height * 0.5), 15.0, f_height)),
            Box::new(BoxBody::new(
                v2!(f_width - 10.0, f_height * 0.5),
                15.0,
                f_height,
            )),
            Box::new(BoxBody::new(v2!(f_width * 0.5, 10.0), f_width, 15.0)),
            Box::new(BoxBody::new(
                v2!(f_width * 0.5, f_height - 10.0),
                f_width,
                15.0,
            )),
            Box::new(TriangleBody::new(
                v2!(f_width * 0.5, 200.0),
                v2!(f_width * 0.5 - 100.0, 300.0),
                v2!(f_width * 0.5 + 100.0, 300.0),
            )),
        ];

        Game {
            time_step: 0.01,
            step_division: 2,
            fluid_system: sph,
            is_simulating: true,
            bodies,

            gameview_offset: Vector2::zero(),
            gameview_width: f_width,
            gameview_height: f_height,
            renderer: Box::new(
                MarchingSquaresRenderer::new(
                    width,
                    height,
                    renderer_step_size,
                    renderer_step_size * 1.5,
                    0.3,
                )
                .unwrap(),
            ),
            draw_particles: false,
        }
    }

    pub fn handle_input(&mut self) {
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = mouse_position();
            let new_particle = Particle::new(Vector2::new(mouse_pos.0, mouse_pos.1))
                .with_color(Color::rgb(0, 0, 255));
            self.fluid_system.add_particle(new_particle);
        }
        if is_mouse_button_down(MouseButton::Right) {
            let mouse_pos = mouse_position();
            let new_particle = Particle::new(Vector2::new(mouse_pos.0, mouse_pos.1))
                .with_color(Color::rgb(255, 0, 0));
            self.fluid_system.add_particle(new_particle);
        }

        // Pause / Resume
        if is_key_pressed(KeyCode::Space) {
            self.is_simulating = !self.is_simulating;
        }
    }

    /// Performs a single update of the game. Should correspond to a single frame.
    pub fn update(&mut self) {
        if self.is_simulating {
            let dt = self.time_step / self.step_division as f32;
            for _ in 0..self.step_division {
                self.fluid_system.step(dt, &self.bodies);
            }
        }

        // Setup graphics
        self.renderer.setup(&self.fluid_system);
    }

    pub fn draw(&self) {
        clear_background(Color::rgb(120, 120, 120).as_mq());
        self.renderer.draw();
        for body in &self.bodies {
            body.draw();
        }

        // Draw individual particles as circles
        if self.draw_particles {
            for p in &self.fluid_system.particles {
                draw_circle(
                    p.position.x,
                    p.position.y,
                    2.0,
                    Color::rgb(255, 255, 255).as_mq(),
                );
            }
        }
    }

    fn is_in_gameview(&self, position: Vector2<f32>) -> bool {
        let relative = position - self.gameview_offset;

        relative.x >= 0.0
            && relative.x < self.gameview_width
            && relative.y >= 0.0
            && relative.y < self.gameview_height
    }
}
