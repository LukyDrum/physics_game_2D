use std::f32::consts::PI;

use macroquad::{
    input::{
        is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position, KeyCode,
        MouseButton,
    },
    shapes::draw_circle,
    window::clear_background,
};

use crate::{
    math::{v2, Vector2},
    physics::rigidbody::{Body, BodyBehaviour, Polygon, RbSimulator, Rectangle},
    rendering::{Color, Draw, MarchingSquaresRenderer, Renderer},
    serialization::SerializationForm,
    utility::AsMq,
    Particle, Sph,
};

use super::{
    config::GameConfig, gamebody::GameBody, save_load, EntityInfo, InGameUI, SaveLoadAction, Tool,
};

pub struct Game {
    game_config: GameConfig,

    pub(crate) fluid_system: Sph,
    /// If the physics are currently being simulated or not
    is_simulating: bool,

    rb_simulator: RbSimulator,
    pub(crate) bodies: Vec<Box<dyn GameBody>>,

    // GUI things
    gameview_offset: Vector2<f32>,
    pub(crate) gameview_width: f32,
    pub(crate) gameview_height: f32,
    renderer: Box<dyn Renderer>,
    draw_particles: bool,
    ingame_ui: InGameUI,
    preview_body: Box<dyn GameBody>,
    mouse_in_gameview: bool,
    pub(crate) name: String,
    pub(crate) description: String,
}

impl Game {
    /// Creates a new instance of Game with all the system instantiated.
    /// `width` and `height` are the dimensions of the game view / game world.
    pub fn new(width: usize, height: usize) -> Self {
        let (f_width, f_height) = (width as f32, height as f32);

        let sph = Sph::new(f_width, f_height);
        let renderer_step_size = f_width / 100.0;

        // Add rectangles that act as walls
        let wall_thickness = 20.0;
        let bodies: Vec<Box<dyn GameBody>> = vec![
            // Floor
            Box::new(
                Rectangle!(v2!(f_width * 0.5, f_height - wall_thickness * 0.5); f_width, wall_thickness; BodyBehaviour::Static),
            ),
            // Ceiling
            Box::new(
                Rectangle!(v2!(f_width * 0.5, wall_thickness * 0.5); f_width, wall_thickness; BodyBehaviour::Static),
            ),
            // Left wall
            Box::new(
                Rectangle!(v2!(wall_thickness * 0.5, f_height * 0.5); wall_thickness, f_height; BodyBehaviour::Static),
            ),
            // Right wall
            Box::new(
                Rectangle!(v2!(f_width - wall_thickness * 0.5, f_height * 0.5); wall_thickness, f_height; BodyBehaviour::Static),
            ),
        ];

        let mut ingame_ui = InGameUI::default();
        ingame_ui
            .body_maker
            .set_max_size(f_width.min(f_height) * 0.8);

        let mut game = Game {
            game_config: GameConfig::default(),

            fluid_system: sph,
            is_simulating: true,

            rb_simulator: RbSimulator::new(v2!(0.0, 981.0)),
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
            ingame_ui,
            preview_body: Box::new(Rectangle!(v2!(50.0, 50.0); 50.0, 50.0; BodyBehaviour::Dynamic)),
            mouse_in_gameview: false,
            name: String::new(),
            description: String::new(),
        };

        game.preview_body = game.body_from_body_maker(v2!(50.0, 50.0));

        game
    }

    fn body_from_body_maker(&self, position: Vector2<f32>) -> Box<dyn GameBody> {
        let size = self.ingame_ui.body_maker.size();
        let orientation = self.ingame_ui.body_maker.orientation();
        let mass = self.ingame_ui.body_maker.mass();
        let mut color = self.ingame_ui.body_maker.color();
        let behaviour = self.ingame_ui.body_maker.behaviour();

        // Create body and set state values
        let mut body: Box<dyn GameBody> = Box::new(Rectangle!(position; size.x, size.y; behaviour));
        body.state_mut().orientation = orientation * (PI / 180.0);
        body.state_mut().set_mass(mass);
        color.a = 0.5;
        body.state_mut().color = color;

        body
    }

    pub fn handle_input(&mut self) {
        let mouse_pos = mouse_position();
        let position = Vector2::new(mouse_pos.0, mouse_pos.1);
        self.mouse_in_gameview = self.is_in_gameview(position);

        match self.ingame_ui.selected_tool {
            Tool::Fluid => {
                if is_mouse_button_down(MouseButton::Left) && self.mouse_in_gameview {
                    self.add_fluid(position);
                }
            }
            Tool::Rigidbody => {
                if self.ingame_ui.body_maker.changed() {
                    self.preview_body = self.body_from_body_maker(position);
                }

                if is_mouse_button_pressed(MouseButton::Left) && self.mouse_in_gameview {
                    let new_body = self.body_from_body_maker(position);

                    let mut body = std::mem::replace(&mut self.preview_body, new_body);
                    // Set color alpha to 1.0 - it was lowered for preview
                    body.state_mut().color.a = 1.0;

                    self.bodies.push(body);
                } else if self.mouse_in_gameview {
                    self.preview_body.set_position(position);
                }
            }
            _ => {}
        }

        // Pause / Resume
        if is_key_pressed(KeyCode::Space) {
            self.is_simulating = !self.is_simulating;
            self.ingame_ui.info_panel.is_simulating = self.is_simulating;
        }
    }

    /// Performs a single update of the game. Should correspond to a single frame.
    pub fn physics_update(&mut self) {
        if self.is_simulating {
            let dt = self.game_config.time_step / self.game_config.sub_steps as f32;

            for _ in 0..self.game_config.sub_steps {
                let fluid_forces_on_bodies =
                    self.fluid_system.step(&self.bodies, &self.game_config, dt);
                for (index, force_accumulation) in fluid_forces_on_bodies {
                    let state = self.bodies[index].state_mut();
                    state.add_force_accumulation(force_accumulation);
                    state.apply_accumulated_forces(dt);
                }

                self.rb_simulator
                    .step(&mut self.bodies, &self.game_config, dt);
            }
        }

        // Setup graphics
        self.renderer.setup(&self.fluid_system);

        // Pass infos to InGameUI
        self.ingame_ui.info_panel.particle_count = self.fluid_system.particle_count();
        self.ingame_ui.info_panel.body_count = self.bodies.len();

        // Find under mouse entity
        let mouse_pos = {
            let (x, y) = mouse_position();
            v2!(x, y)
        };

        let mut entity_info = EntityInfo::Nothing {
            position: mouse_pos,
        };
        for body in &self.bodies {
            if body.contains_point(mouse_pos) {
                entity_info = EntityInfo::Body {
                    position: body.state().position,
                    velocity: body.state().velocity,
                    mass: body.state().mass(),
                    color: Color::rgb(0, 0, 0),
                };
                break;
            }
        }
        if let EntityInfo::Nothing { .. } = entity_info {
            if let Some((_, closest_p)) = self
                .fluid_system
                .get_particles_around_position(mouse_pos, 10.0)
                .into_iter()
                .map(|p| ((p.position - mouse_pos).length_squared(), p))
                .min_by(|a, b| a.0.total_cmp(&b.0))
            {
                entity_info = EntityInfo::Fluid {
                    position: closest_p.position,
                    velocity: closest_p.velocity,
                    density: closest_p.mass(),
                    color: closest_p.color,
                };
            }
        }

        self.ingame_ui.info_panel.under_mouse_entity = entity_info;
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

    pub fn draw_ui(&mut self) {
        self.ingame_ui.draw(
            Vector2::new(self.gameview_width + 50.0, 40.0),
            &mut self.game_config,
        );

        if let Tool::Rigidbody = self.ingame_ui.selected_tool {
            if self.mouse_in_gameview {
                self.preview_body.draw();
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

    fn add_fluid(&mut self, position: Vector2<f32>) {
        let particle = Particle::new(position)
            .with_mass(self.ingame_ui.fluid_selector.density())
            .with_color(self.ingame_ui.fluid_selector.color());
        self.fluid_system.add_particle(particle);
    }

    fn handle_save_loads(&mut self) {
        let save_file_name = self.ingame_ui.save_loads.save_file_name.as_str();
        match std::mem::replace(
            &mut self.ingame_ui.save_loads.action,
            SaveLoadAction::Nothing,
        ) {
            SaveLoadAction::Save if !save_file_name.is_empty() => {
                save_load::save(self.to_serialized_form(), save_file_name)
            }
            SaveLoadAction::Load(game_serialized_form) => {
                let mut new_game = Game::from_serialized_form(game_serialized_form);

                // Swap things that should not change
                std::mem::swap(&mut self.ingame_ui, &mut new_game.ingame_ui);

                *self = new_game;
            }
            _ => {}
        }
    }

    fn handle_tool_change_keys(&mut self) {
        if self.ingame_ui.save_loads.taken_input {
            return;
        }

        if is_key_pressed(KeyCode::I) {
            self.ingame_ui.selected_tool = Tool::Info;
        } else if is_key_pressed(KeyCode::F) {
            self.ingame_ui.selected_tool = Tool::Fluid;
        } else if is_key_pressed(KeyCode::B) {
            self.ingame_ui.selected_tool = Tool::Rigidbody;
        } else if is_key_pressed(KeyCode::C) {
            self.ingame_ui.selected_tool = Tool::Configuration;
        } else if is_key_pressed(KeyCode::L) {
            self.ingame_ui.selected_tool = Tool::SaveLoads;
        }
    }

    pub fn update(&mut self) {
        self.handle_input();
        self.physics_update();
        self.draw();
        self.draw_ui();
        self.handle_save_loads();
        self.handle_tool_change_keys();
    }
}
