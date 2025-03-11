use crate::{math::Vector2, Sph};

use macroquad::prelude::*;

use super::renderer::Renderer;
use super::{Color, SamplePoint};

pub struct ScalarFieldRenderer {
    scalar_field: Vec<SamplePoint>,
    field_width: usize,
    field_height: usize,
    step_size: f32,
    influence_radius: f32,
    draw_threshold: f32,
}

impl ScalarFieldRenderer {
    /// Returns error if `screen_width` or `screen_height` are not multiple of the `step_size`.
    pub fn new(
        screen_width: usize,
        screen_height: usize,
        step_size: f32,
        influence_radius: f32,
    ) -> Result<Self, ()> {
        if screen_width as f32 % step_size != 0.0 || screen_height as f32 % step_size != 0.0 {
            return Err(());
        }

        let field_width = (screen_width as f32 / step_size) as usize + 1;
        let field_height = (screen_height as f32 / step_size) as usize + 1;

        Ok(ScalarFieldRenderer {
            scalar_field: vec![SamplePoint::default(); field_width * field_height],
            field_width,
            field_height,
            step_size,
            influence_radius,
            draw_threshold: 0.8,
        })
    }

    fn index_to_position(&self, i: usize) -> Vector2<f32> {
        let x = (i % self.field_height) as f32 * self.step_size;
        let y = (i / self.field_width) as f32 * self.step_size;
        Vector2::new(x, y)
    }
}

impl Renderer for ScalarFieldRenderer {
    fn setup(&mut self, sph: &Sph) {
        for i in 0..(self.field_width * self.field_height) {
            let pos = self.index_to_position(i);

            let particles = sph.get_particles_around_position(pos, self.influence_radius);
            let sample = particles
                .iter()
                .enumerate()
                .map(|(index, p)| {
                    let dist = (p.position - pos).length();
                    (index, (self.step_size / dist, p.color))
                })
                .fold(
                    SamplePoint::default(),
                    |mut acc, (index, (value, color))| {
                        acc.scalar_value += value;
                        let r = (index as f32 * acc.color.r + color.r) / (index as f32 + 1.0);
                        let g = (index as f32 * acc.color.g + color.g) / (index as f32 + 1.0);
                        let b = (index as f32 * acc.color.b + color.b) / (index as f32 + 1.0);
                        acc.color = Color::new(r, g, b, 1.0); // Make the color always max alpha

                        acc
                    },
                );
            self.scalar_field[i] = sample;
        }
    }

    fn draw(&self) {
        for i in 0..(self.field_width * self.field_height) {
            if self.scalar_field[i].scalar_value < self.draw_threshold {
                continue;
            }
            let pos = self.index_to_position(i);
            // Make the color of the 'pixel' relative to the concentration there
            let mut color = self.scalar_field[i].color;
            color.a = (self.scalar_field[i].scalar_value / self.influence_radius).min(1.0);
            draw_rectangle(
                pos.x - self.step_size * 0.5,
                pos.y - self.step_size * 0.5,
                self.step_size,
                self.step_size,
                color.as_mq(),
            );
        }
    }
}
