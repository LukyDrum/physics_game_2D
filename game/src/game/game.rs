use std::{collections::LinkedList, f32::consts::PI};

use macroquad::{
    input::{
        is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released,
        mouse_position, KeyCode, MouseButton,
    },
    shapes::draw_circle,
    text::draw_text,
    window::clear_background,
};

use crate::{
    math::{v2, Vector2},
    physics::rigidbody::{BodyBehaviour, RbSimulator, Rectangle, RigidBody, SharedProperty},
    rendering::{Color, Draw, MarchingSquaresRenderer, Renderer},
    serialization::{GameSerializedForm, SerializationForm},
    utility::AsMq,
    Particle, Sph,
};

use super::{
    config::GameConfig, save_load, EntityInfo, FluidSelectorAction, InGameUI, QuickAction,
    SaveLoadAction, Tool, FONT_SIZE_LARGE, FONT_SIZE_SMALL,
};

struct DraggedBody {
    pub index: usize,
    pub drag_offset: Vector2<f32>,
}

pub struct Game {
    game_config: GameConfig,

    pub quit_flag: bool,
    pub(crate) save_name: String,

    pub(crate) fluid_system: Sph,
    /// If the physics are currently being simulated or not
    is_simulating: bool,

    pub(crate) rb_simulator: RbSimulator,

    // GUI things
    gameview_offset: Vector2<f32>,
    pub(crate) gameview_width: f32,
    pub(crate) gameview_height: f32,
    renderer: Box<dyn Renderer>,
    draw_particles: bool,
    ingame_ui: InGameUI,
    preview_body: RigidBody,
    mouse_in_gameview: bool,
    pub(crate) name: String,
    pub(crate) description: LinkedList<String>,

    mouse_position_last_frame: Vector2<f32>,
    dragged_body: Option<DraggedBody>,
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
        let mut bodies = vec![
            // Floor
            Rectangle!(v2!(f_width * 0.5, f_height - wall_thickness * 0.5); f_width, wall_thickness; BodyBehaviour::Static),
            // Ceiling
            Rectangle!(v2!(f_width * 0.5, wall_thickness * 0.5); f_width, wall_thickness; BodyBehaviour::Static),
            // Left wall
            Rectangle!(v2!(wall_thickness * 0.5, f_height * 0.5); wall_thickness, f_height; BodyBehaviour::Static),
            // Right wall
            Rectangle!(v2!(f_width - wall_thickness * 0.5, f_height * 0.5); wall_thickness, f_height; BodyBehaviour::Static),
        ];
        // Set shared properties to pass
        for body in &mut bodies {
            let state = body.state_mut();
            state.elasticity = SharedProperty::Pass;
            state.static_friction = SharedProperty::Pass;
            state.dynamic_friction = SharedProperty::Pass;
        }

        let mut ingame_ui = InGameUI::default();
        ingame_ui.body_maker.set_max_size(f_width.min(f_height));

        let mut rb_simulator = RbSimulator::new(v2!(0.0, 981.0));
        rb_simulator.bodies = bodies;

        let mut game = Game {
            game_config: GameConfig::default(),

            quit_flag: false,
            save_name: "_Default".to_string(),

            fluid_system: sph,
            is_simulating: true,

            rb_simulator,

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
            preview_body: Rectangle!(v2!(50.0, 50.0); 50.0, 50.0; BodyBehaviour::Dynamic),
            mouse_in_gameview: false,
            name: String::new(),
            description: LinkedList::new(),

            mouse_position_last_frame: Vector2::zero(),
            dragged_body: None,
        };

        game.preview_body = game.body_from_body_maker(v2!(50.0, 50.0));

        game
    }

    pub(crate) fn set_description(&mut self, description: String) {
        const MAX_WORDS: usize = 10;

        self.description = description
            .split("\n")
            .map(|s| s.to_owned())
            .flat_map(|line| {
                let split = line.split(" ").collect::<Vec<_>>();
                if split.len() > MAX_WORDS {
                    let start = split[..MAX_WORDS].join(" ");
                    let end = split[MAX_WORDS..].join(" ");
                    vec![start, end]
                } else {
                    vec![line]
                }
            })
            .collect();
    }

    fn body_from_body_maker(&self, position: Vector2<f32>) -> RigidBody {
        let body_maker = &self.ingame_ui.body_maker;
        let size = body_maker.size();
        let orientation = body_maker.orientation;
        let lock_rotation = body_maker.lock_rotation;
        let mass = body_maker.mass;
        let mut color = body_maker.color();
        let behaviour = body_maker.behaviour;
        let elasticity = body_maker.elasticity;
        let static_friction = body_maker.static_friction;
        let dynamic_friction = body_maker.dynamic_friction;

        // Create body and set state values
        let mut body = Rectangle!(position; size.x, size.y; behaviour);
        body.state_mut().orientation = orientation * (PI / 180.0);
        body.state_mut().lock_rotation = lock_rotation;
        body.state_mut().set_mass(mass);
        color.a = 0.5;
        body.state_mut().color = color;
        body.state_mut().elasticity = SharedProperty::Value(elasticity);
        body.state_mut().static_friction = SharedProperty::Value(static_friction);
        body.state_mut().dynamic_friction = SharedProperty::Value(dynamic_friction);

        body
    }

    pub fn handle_input(&mut self) {
        let mouse_pos = mouse_position();
        let position = Vector2::new(mouse_pos.0, mouse_pos.1);
        self.mouse_in_gameview = self.is_in_gameview(position);

        // Release dragged body
        if is_mouse_button_released(MouseButton::Left) && self.dragged_body.is_some() {
            self.dragged_body = None;
        }

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

                // Set dragged body by holding left mouse button on it
                if is_mouse_button_down(MouseButton::Left) && self.dragged_body.is_none() {
                    if let EntityInfo::Body {
                        index,
                        position: body_position,
                        ..
                    } = self.ingame_ui.info_panel.under_mouse_entity
                    {
                        if index >= 4 {
                            self.dragged_body = Some(DraggedBody {
                                index,
                                drag_offset: position - body_position,
                            });
                        }
                    }
                }
                // Move dragged body
                if let Some(DraggedBody { index, drag_offset }) = self.dragged_body {
                    let state = self.rb_simulator.bodies[index].state_mut();
                    let position = position.clamp(
                        v2!(0.0, 0.0),
                        v2!(self.gameview_width, self.gameview_height),
                    );
                    match state.behaviour {
                        BodyBehaviour::Dynamic => {
                            let pos_diff = position - state.position - drag_offset;
                            state.velocity = pos_diff * 10.0;
                        }
                        BodyBehaviour::Static => {
                            let new_pos = position - drag_offset;
                            self.rb_simulator.bodies[index].set_position(new_pos);
                        }
                    }
                }

                // Spawn bodies with right click
                if is_mouse_button_pressed(MouseButton::Right) && self.mouse_in_gameview {
                    let new_body = self.body_from_body_maker(position);

                    let mut body = std::mem::replace(&mut self.preview_body, new_body);
                    // Set color alpha to 1.0 - it was lowered for preview
                    body.state_mut().color.a = 1.0;

                    self.rb_simulator.bodies.push(body);
                }
                // Delete bodies with middle click
                else if is_mouse_button_pressed(MouseButton::Middle) {
                    if let EntityInfo::Body { index, .. } =
                        self.ingame_ui.info_panel.under_mouse_entity
                    {
                        // Do not remove the first 4 bodies - those are walls
                        if index >= 4 {
                            self.rb_simulator.bodies.swap_remove(index);
                        }
                    }
                } else if self.mouse_in_gameview {
                    self.preview_body.set_position(position);
                }
            }
            _ => {}
        }

        // Pause / Resume
        if is_key_pressed(KeyCode::Space) {
            self.toggle_pause();
        }

        // Set new mouse last pos
        self.mouse_position_last_frame = position;
    }

    fn toggle_pause(&mut self) {
        self.is_simulating = !self.is_simulating;
        self.ingame_ui.info_panel.is_simulating = self.is_simulating;
    }

    /// Performs a single update of the game. Should correspond to a single frame.
    pub fn physics_update(&mut self) {
        if self.is_simulating {
            let dt = self.game_config.time_step / self.game_config.sub_steps as f32;

            for _ in 0..self.game_config.sub_steps {
                let fluid_forces_on_bodies =
                    self.fluid_system
                        .step(&self.rb_simulator.bodies, &self.game_config, dt);
                for (index, force_accumulation) in fluid_forces_on_bodies {
                    let state = self.rb_simulator.bodies[index].state_mut();
                    state.add_force_accumulation(force_accumulation);
                    state.apply_accumulated_forces(dt);
                }

                self.rb_simulator.step(&self.game_config, dt);
            }
        }

        // Setup graphics
        self.renderer.setup(&self.fluid_system);

        // Pass infos to InGameUI
        self.ingame_ui.info_panel.particle_count = self.fluid_system.particle_count();
        self.ingame_ui.info_panel.body_count = self.rb_simulator.bodies.len();

        // Find under mouse entity
        let mouse_pos = {
            let (x, y) = mouse_position();
            v2!(x, y)
        };

        let mut entity_info = EntityInfo::Nothing {
            position: mouse_pos,
        };
        for (index, body) in self.rb_simulator.bodies.iter().enumerate() {
            if body.contains_point(mouse_pos) {
                entity_info = EntityInfo::Body {
                    index,
                    position: body.state().position,
                    velocity: body.state().velocity,
                    mass: body.state().mass(),
                    color: body.state().color,
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
        for body in &self.rb_simulator.bodies {
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
            if self.mouse_in_gameview && self.dragged_body.is_none() {
                self.preview_body.draw();
            }
        }

        if let Tool::Fluid = self.ingame_ui.selected_tool {
            if let FluidSelectorAction::ClearParticles = self.ingame_ui.fluid_selector.action {
                self.fluid_system.clear_all_particles();
            }
        }

        // Draw name and description text
        let offset = v2!(30.0, self.gameview_height + 30.0);
        draw_text(
            &self.name,
            offset.x,
            offset.y,
            FONT_SIZE_LARGE,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let mut offset = offset + v2!(0.0, 30.0);
        for line in &self.description {
            draw_text(
                line,
                offset.x,
                offset.y,
                FONT_SIZE_SMALL,
                Color::rgb(0, 0, 0).as_mq(),
            );
            offset.y += FONT_SIZE_SMALL + 5.0;
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
        let fluid_tool = &self.ingame_ui.fluid_selector;
        let droplet_count = fluid_tool.droplet_count;
        let mass = fluid_tool.density;
        let color = fluid_tool.color();

        for _ in 0..droplet_count {
            let x_off = 2.0 * fastrand::f32() - 1.0;
            let y_off = 2.0 * fastrand::f32() - 1.0;
            let position = position + v2!(x_off, y_off);

            let particle = Particle::new(position).with_mass(mass).with_color(color);
            self.fluid_system.add_particle(particle);
        }
    }

    fn handle_save_loads(&mut self) {
        let save_file_name = self.ingame_ui.save_loads.save_file_name.clone();
        match std::mem::replace(
            &mut self.ingame_ui.save_loads.action,
            SaveLoadAction::Nothing,
        ) {
            SaveLoadAction::Save if !save_file_name.is_empty() => {
                let mut ser = self.to_serialized_form();
                ser.name = save_file_name.clone();
                ser.description = "".to_string();

                save_load::save(self.to_serialized_form(), save_file_name.as_str());
                self.save_name = save_file_name.to_string();
            }
            SaveLoadAction::Load(game_serialized_form) => {
                *self = self.prepared_load_game(game_serialized_form);
            }
            _ => {}
        }
    }

    fn prepared_load_game(&mut self, ser_form: GameSerializedForm) -> Game {
        let mut new_game = Game::from_serialized_form(ser_form);

        // Swap things that should not change
        std::mem::swap(&mut self.ingame_ui, &mut new_game.ingame_ui);
        std::mem::swap(&mut self.preview_body, &mut new_game.preview_body);

        new_game
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

    fn handle_quick_menu_actions(&mut self) {
        match self.ingame_ui.quick_menu.action {
            QuickAction::Quit => self.quit_flag = true,
            QuickAction::Restart => {
                *self = self.prepared_load_game(save_load::load_save(self.save_name.as_str()));
            }
            QuickAction::TogglePause => self.toggle_pause(),
            QuickAction::Nothing => {}
        }
    }

    pub fn update(&mut self) {
        self.handle_input();
        self.physics_update();
        self.draw();
        self.draw_ui();

        // Handle UI events
        self.handle_quick_menu_actions();
        self.handle_save_loads();
        self.handle_tool_change_keys();
    }
}
