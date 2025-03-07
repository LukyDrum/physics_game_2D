use crate::{math::Vector2, Sph};

use macroquad::prelude::*;

use super::renderer::Renderer;

pub struct ScalarFieldRenderer {
    scalar_field: Vec<f32>,
    field_width: usize,
    field_height: usize,
    step_size: f32,
    draw_threshold: f32,
}

impl ScalarFieldRenderer {
    /// Returns error if `screen_width` or `screen_height` are not multiple of the `step_size`.
    pub fn new(screen_width: usize, screen_height: usize, step_size: f32) -> Result<Self, ()> {
        if screen_width as f32 % step_size != 0.0 || screen_height as f32 % step_size != 0.0 {
            return Err(());
        }

        let field_width = (screen_width as f32 / step_size) as usize;
        let field_height = (screen_height as f32 / step_size) as usize;

        Ok(ScalarFieldRenderer {
            scalar_field: vec![0f32; field_width * field_height],
            field_width,
            field_height,
            step_size: step_size as f32,
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

            let particles = sph.get_particles_around_position(pos, self.step_size);
            let value = particles
                .iter()
                .map(|p| {
                    let dist = (p.position - pos).length();
                    self.step_size / dist
                })
                .sum();
            self.scalar_field[i] = value;
        }
    }

    fn draw(&self) {
        for i in 0..(self.field_width * self.field_height) {
            if self.scalar_field[i] < self.draw_threshold {
                continue;
            }
            let pos = self.index_to_position(i);
            draw_rectangle(pos.x - self.step_size, pos.y - self.step_size, 2.0 * self.step_size, 2.0 * self.step_size, BLUE);
        }
    }
}
