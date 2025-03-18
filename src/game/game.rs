use macroquad::{
    input::{is_mouse_button_down, mouse_position, MouseButton}, shapes::draw_circle, window::clear_background
};

use crate::{
    math::{v2, Vector2},
    physics::rigidbody::{Body, Container, Line, RBox, Triangle},
    rendering::{Color, Draw, Renderer, ScalarFieldRenderer},
    Particle, SimulationConfig, Sph,
};

pub trait GameBody: Body + Draw + Sync {}
impl GameBody for Line {}
impl GameBody for RBox {}
impl GameBody for Triangle {}
impl GameBody for Container {}

pub struct Game {
    // Physics stuff
    time_step: f32,
    fluid_system: Sph,

    bodies: Vec<Box<dyn GameBody>>,

    // GUI things
    gameview_offset: Vector2<f32>,
    gameview_width: f32,
    gameview_height: f32,
    renderer: Box<dyn Renderer>,
}

impl Game {
    /// Creates a new instance of Game with all the system instantiated.
    /// `width` and `height` are the dimensions of the game view / game world.
    pub fn new(width: usize, height: usize) -> Self {
        let (f_width, f_height) = (width as f32, height as f32);

        let sph = Sph::new(SimulationConfig::default(), f_width, f_height);
        let renderer_step_size = f_width / 100.0;
        // Add basic container
        let bodies: Vec<Box<dyn GameBody>> = vec![
            Box::new(Container::new(v2!(f_width * 0.5, f_height * 0.5), f_width, f_height)),
        ];

        Game {
            time_step: 0.01,
            fluid_system: sph,
            bodies,

            gameview_offset: Vector2::zero(),
            gameview_width: f_width,
            gameview_height: f_height,
            renderer: Box::new(
                ScalarFieldRenderer::new(width, height, renderer_step_size, renderer_step_size)
                    .unwrap(),
            ),
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
    }

    pub fn update(&mut self) {
        self.fluid_system.step(self.time_step, &self.bodies);

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
        for p in &self.fluid_system.particles {
            draw_circle(p.position.x, p.position.y, 2.0, Color::rgb(255, 255, 255).as_mq());
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
