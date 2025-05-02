use std::fmt::Display;

use macroquad::{
    text::{draw_text, TextDimensions},
    time::get_fps,
    ui::{root_ui, widgets::Slider},
};

use crate::{
    game::ui::game_ui::FONT_SIZE_MEDIUM,
    game::UIComponent,
    math::{v2, Vector2},
    rendering::Color,
    utility::AsMq,
};

fn draw_vector2(vector: Vector2<f32>, offset: Vector2<f32>, preword: &str) -> TextDimensions {
    let text = format!("{} X: {:.2}, Y: {:.2}", preword, vector.x, vector.y);
    draw_text(
        text.as_str(),
        offset.x,
        offset.y,
        FONT_SIZE_MEDIUM,
        Color::rgb(0, 0, 0).as_mq(),
    )
}

#[derive(Clone, Copy)]
pub enum EntityInfo {
    Nothing {
        position: Vector2<f32>,
    },
    Body {
        position: Vector2<f32>,
        velocity: Vector2<f32>,
        mass: f32,
        color: Color,
    },
    Fluid {
        position: Vector2<f32>,
        velocity: Vector2<f32>,
        density: f32,
        color: Color,
    },
}

impl EntityInfo {
    pub fn draw(&self, offset: Vector2<f32>) {
        match self {
            EntityInfo::Nothing { position } => {
                draw_vector2(*position, offset, "Mouse position:");
            }
            EntityInfo::Body {
                position,
                velocity,
                mass,
                color,
            } => {
                let dim = draw_vector2(*position, offset, "Position:");

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let dim = draw_vector2(*velocity, offset, "Velocity:");

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let dim = draw_text(
                    format!("Mass: {:.2} [g]", mass).as_str(),
                    offset.x,
                    offset.y,
                    FONT_SIZE_MEDIUM,
                    Color::rgb(0, 0, 0).as_mq(),
                );

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let _dim = draw_text(
                    format!(
                        "Color: ({}, {}, {})",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    )
                    .as_str(),
                    offset.x,
                    offset.y,
                    FONT_SIZE_MEDIUM,
                    Color::rgb(0, 0, 0).as_mq(),
                );
            }
            EntityInfo::Fluid {
                position,
                velocity,
                density,
                color,
            } => {
                let dim = draw_vector2(*position, offset, "Position:");

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let dim = draw_vector2(*velocity, offset, "Velocity:");

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let dim = draw_text(
                    format!("Density: {:.2} [g/cm^3]", density).as_str(),
                    offset.x,
                    offset.y,
                    FONT_SIZE_MEDIUM,
                    Color::rgb(0, 0, 0).as_mq(),
                );

                let offset = offset + v2!(0.0, dim.height + 20.0);
                let _dim = draw_text(
                    format!(
                        "Color: ({}, {}, {})",
                        (color.r * 255.0) as u8,
                        (color.g * 255.0) as u8,
                        (color.b * 255.0) as u8
                    )
                    .as_str(),
                    offset.x,
                    offset.y,
                    FONT_SIZE_MEDIUM,
                    Color::rgb(0, 0, 0).as_mq(),
                );
            }
        }
    }
}

pub struct InfoPanel {
    pub particle_count: usize,
    pub body_count: usize,
    pub under_mouse_entity: EntityInfo,
    pub is_simulating: bool,
}

impl Default for InfoPanel {
    fn default() -> Self {
        InfoPanel {
            particle_count: 0,
            body_count: 0,
            under_mouse_entity: EntityInfo::Nothing {
                position: Vector2::zero(),
            },
            is_simulating: true,
        }
    }
}

impl UIComponent for InfoPanel {
    fn draw(&mut self, offset: Vector2<f32>) {
        let offset = offset + v2!(0.0, 20.0);
        let fps = if self.is_simulating {
            format!("FPS: {}", get_fps())
        } else {
            format!("FPS: (paused)")
        };
        let dim = draw_text(
            fps.as_str(),
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let offset = offset + v2!(0.0, dim.height + 20.0);
        let p_count = format!("Particle count: {}", self.particle_count);
        let dim = draw_text(
            p_count.as_str(),
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let offset = offset + v2!(0.0, dim.height + 20.0);
        let body_count = format!("Body count: {}", self.body_count);
        let dim = draw_text(
            body_count.as_str(),
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let offset = offset + v2!(0.0, dim.height + 40.0);
        let entity_name = match self.under_mouse_entity {
            EntityInfo::Nothing { .. } => "Nothing",
            EntityInfo::Fluid { .. } => "Fluid particle",
            EntityInfo::Body { .. } => "Body",
        };
        let dim = draw_text(
            format!("Under-cursor: {entity_name}").as_str(),
            offset.x,
            offset.y,
            FONT_SIZE_MEDIUM,
            Color::rgb(0, 0, 0).as_mq(),
        );

        let offset = offset + v2!(20.0, dim.height + 20.0);
        self.under_mouse_entity.draw(offset);
    }
}
