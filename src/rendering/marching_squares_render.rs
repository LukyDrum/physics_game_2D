use crate::{math::Vector2, Sph};

use macroquad::prelude::*;
use num_traits::Pow;

use super::renderer::Renderer;

/// Alias for a tuple of 2 Vector2.
/// They represent the start and end of a line.
type Line<T> = (Vector2<T>, Vector2<T>);

const fn line(a_x: f32, a_y: f32, b_x: f32, b_y: f32) -> Line<f32> {
    (Vector2::new(a_x, a_y), Vector2::new(b_x, b_y))
}

/// Returns all possible configurations of corners with the lines that should be drawn.
/// Top-left corner is (0,0), bottom-right is (1,1)
fn configurations() -> [Vec<Line<f32>>; 16] {
    [
        vec![],                                                   // 0b0000 = 0
        vec![line(0.0, 0.5, 0.5, 1.0)],                           // 0b0001 = 1
        vec![line(0.5, 1.0, 1.0, 0.5)],                           // 0b0010 = 2
        vec![line(0.0, 0.5, 1.0, 0.5)],                           // 0b0011 = 3
        vec![line(0.5, 0.0, 1.0, 0.5)],                           // 0b0100 = 4
        vec![line(0.5, 0.0, 1.0, 0.5), line(0.0, 0.5, 0.5, 1.0)], // 0b0101 = 5
        vec![line(0.5, 0.0, 0.5, 1.0)],                           // 0b0110 = 6
        vec![line(0.0, 0.5, 0.5, 0.0)],                           // 0b0111 = 7
        vec![line(0.5, 0.0, 0.0, 0.5)],                           // 0b1000 = 8
        vec![line(0.5, 0.0, 0.5, 1.0)],                           // 0b1001 = 9
        vec![line(0.0, 0.5, 0.5, 0.0), line(0.5, 1.0, 1.0, 0.5)], // 0b1010 = 10
        vec![line(0.5, 0.0, 1.0, 0.5)],                           // 0b1011 = 11
        vec![line(0.0, 0.5, 1.0, 0.5)],                           // 0b1100 = 12
        vec![line(0.5, 1.0, 1.0, 0.5)],                           // 0b1101 = 13
        vec![line(0.0, 0.5, 0.5, 1.0)],                           // 0b1110 = 14
        vec![],                                                   // 0b1111 = 15
    ]
}

pub struct MarchingSquaresRenderer {
    scalar_field: Vec<f32>,
    field_width: usize,
    field_height: usize,
    step_size: f32,
    influence_radius: f32,
    draw_threshold: f32,
    configurations: [Vec<Line<f32>>; 16],
}

impl MarchingSquaresRenderer {
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

        Ok(MarchingSquaresRenderer {
            scalar_field: vec![0f32; field_width * field_height],
            field_width,
            field_height,
            step_size,
            influence_radius,
            draw_threshold: 0.5,
            configurations: configurations(),
        })
    }

    fn index_to_position(&self, i: usize) -> Vector2<f32> {
        let x = (i % self.field_height) as f32 * self.step_size;
        let y = (i / self.field_width) as f32 * self.step_size;
        Vector2::new(x, y)
    }

    fn configuration_number(&self, i: usize) -> usize {
        // We know that `i` will always be a valid index
        let top_left = self.scalar_field[i];
        // We try the rest and always choose the previouse one if it is out of bounds
        let top_right = *self.scalar_field.get(i + 1).unwrap_or(&top_left);
        let bottom_left = *self
            .scalar_field
            .get(i + self.field_width)
            .unwrap_or(&top_right);
        let bottom_right = *self
            .scalar_field
            .get(i + self.field_width + 1)
            .unwrap_or(&bottom_left);

        let mut conf_number = 0;
        // Exact order we need to iterate in
        for (i, val) in [top_left, top_right, bottom_right, bottom_left]
            .iter()
            .enumerate()
        {
            if *val >= self.draw_threshold {
                // We need to go from the highest power to lowest
                conf_number += 2.pow(3 - i) as usize;
            }
        }

        conf_number
    }
}

impl Renderer for MarchingSquaresRenderer {
    fn setup(&mut self, sph: &Sph) {
        for i in 0..(self.field_width * self.field_height) {
            let pos = self.index_to_position(i);

            let particles = sph.get_particles_around_position(pos, self.influence_radius);
            let value = particles
                .iter()
                .map(|p| {
                    let dist = (p.position - pos).length();
                    self.influence_radius / dist
                })
                .sum();
            self.scalar_field[i] = value;
        }
    }

    fn draw(&self) {
        for i in 0..(self.field_width * self.field_height) {
            let pos = self.index_to_position(i);

            let conf_number = self.configuration_number(i);
            let conf = &self.configurations[conf_number];
            for line in conf {
                let a = pos + line.0 * self.step_size;
                let b = pos + line.1 * self.step_size;
                draw_line(a.x, a.y, b.x, b.y, 3.0, BLUE);
            }
        }
    }
}
