use core::panic;

use crate::math::v2;
use crate::utility::non_zero_average;
use crate::{math::Vector2, Sph};

use macroquad::prelude::*;
use num_traits::Pow;

use super::renderer::Renderer;
use super::{triangulate_convex_polygon, VectorAsMQ};
use super::{Color, SamplePoint};

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

struct AppliedConfiguration {
    lines: Vec<Line<f32>>,
    configuration_id: usize,
    color: Color,
}

pub struct MarchingSquaresRenderer {
    sample_field: Vec<SamplePoint>,
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
        draw_threshold: f32,
    ) -> Result<Self, ()> {
        let field_width = (screen_width as f32 / step_size) as usize + 1;
        let field_height = (screen_height as f32 / step_size) as usize + 1;

        Ok(MarchingSquaresRenderer {
            sample_field: vec![SamplePoint::default(); field_width * field_height],
            field_width,
            field_height,
            step_size,
            influence_radius,
            draw_threshold,
            configurations: configurations(),
        })
    }

    fn index_to_position(&self, i: usize) -> Vector2<f32> {
        let x = (i % self.field_height) as f32 * self.step_size;
        let y = (i / self.field_width) as f32 * self.step_size;
        Vector2::new(x, y)
    }

    fn configuration_from_corner(&self, i: usize) -> AppliedConfiguration {
        // We know that `i` will always be a valid index
        let top_left = self.sample_field[i].scalar_value;
        // We try the rest and always choose the previouse one if it is out of bounds
        let top_right = self
            .sample_field
            .get(i + 1)
            .map(|s| s.scalar_value)
            .unwrap_or(top_left);
        let bottom_left = self
            .sample_field
            .get(i + self.field_width)
            .map(|s| s.scalar_value)
            .unwrap_or(top_right);
        let bottom_right = self
            .sample_field
            .get(i + self.field_width + 1)
            .map(|s| s.scalar_value)
            .unwrap_or(bottom_left);

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

        let mut conf = self.configurations[conf_number].clone();
        // Linear interpolation for each line
        for (a, b) in &mut conf {
            for point in [a, b] {
                // For x coordinate
                if point.x > 0.0 && point.x < 1.0 {
                    let (start, diff) = match point.y {
                        // Top side
                        0.0 => (top_left, top_left - top_right),
                        // Bottom side
                        1.0 => (bottom_left, bottom_left - bottom_right),
                        _ => continue,
                    };

                    // Set the new interpolated x coordinate
                    if diff > 0.0 {
                        point.x = (self.draw_threshold - start).abs() / diff.abs();
                    }
                }

                // For y coordinate
                if point.y > 0.0 && point.y < 1.0 {
                    let (start, diff) = match point.x {
                        // Left side
                        0.0 => (top_left, top_left - bottom_left),
                        // Right side
                        1.0 => (top_right, top_right - bottom_right),
                        _ => continue,
                    };

                    // Set the new interpolated y coordinate
                    if diff > 0.0 {
                        point.y = (self.draw_threshold - start).abs() / diff.abs();
                    }
                }
            }
        }

        AppliedConfiguration {
            lines: conf,
            configuration_id: conf_number,
            color: self.get_color_from_corner(i),
        }
    }

    fn get_color_from_corner(&self, i: usize) -> Color {
        let (top_left, tl_value) = (
            self.sample_field[i].color,
            self.sample_field[i].scalar_value,
        );
        // We try the rest and always choose the previouse one if it is out of bounds
        let (top_right, tr_value) = self
            .sample_field
            .get(i + 1)
            .map(|s| (s.color, s.scalar_value))
            .unwrap_or((top_left, tl_value));
        let (bottom_left, bl_value) = self
            .sample_field
            .get(i + self.field_width)
            .map(|s| (s.color, s.scalar_value))
            .unwrap_or((top_right, tr_value));
        let (bottom_right, br_value) = self
            .sample_field
            .get(i + self.field_width + 1)
            .map(|s| (s.color, s.scalar_value))
            .unwrap_or((bottom_left, bl_value));

        let average = (tl_value + tr_value + bl_value + br_value) * 0.25;
        // Average the colors in each corner
        let r = non_zero_average(&[top_left.r, top_right.r, bottom_left.r, bottom_right.r], 0.2);
        let g = non_zero_average(&[top_left.g, top_right.g, bottom_left.g, bottom_right.g], 0.2);
        let b = non_zero_average(&[top_left.b, top_right.b, bottom_left.b, bottom_right.b], 0.2);
        let a =
            non_zero_average(&[top_left.a, top_right.a, bottom_left.a, bottom_right.a], 0.2) * average;

        Color::new(r, g, b, a)
    }

    fn local_point(&self, base: Vector2<f32>, offset: Vector2<f32>) -> Vector2<f32> {
        base + offset * self.step_size
    }
}

impl Renderer for MarchingSquaresRenderer {
    fn setup(&mut self, sph: &Sph) {
        let half_step = self.step_size * 0.5;
        for i in 0..(self.field_width * self.field_height) {
            let pos = self.index_to_position(i) + v2!(half_step, half_step);

            let particles = sph.get_particles_around_position(pos, self.influence_radius);

            let sample = particles
                .iter()
                .map(|p| {
                    let dist = (p.position - pos).length();
                    let influence = if dist > self.influence_radius {
                        0.0
                    } else {
                        self.influence_radius / dist
                    };
                    (influence, p.color)
                })
                .fold(SamplePoint::default(), |mut acc, (value, color)| {
                    acc.scalar_value += value;
                    acc.color.r += color.r * value;
                    acc.color.g += color.g * value;
                    acc.color.b += color.b * value;

                    acc
                });

            // Get weighted average of the color
            let color = Color::new(
                sample.color.r / sample.scalar_value,
                sample.color.g / sample.scalar_value,
                sample.color.b / sample.scalar_value,
                1.0,
            );
            
            self.sample_field[i].color = color;
            self.sample_field[i].scalar_value =
                (self.sample_field[i].scalar_value + sample.scalar_value) * 0.5;
        }
    }

    fn draw(&self) {
        for i in 0..(self.field_width * self.field_height) {
            let pos = self.index_to_position(i);
            let conf = self.configuration_from_corner(i);

            // Special cases - if matched then this will jump to next loop iteration
            // Those are: Empty, Full, Opossite corners
            match conf.configuration_id {
                // Empty - draw nothing
                0b0000 => continue,
                // Full - draw a rectangle
                0b1111 => {
                    draw_rectangle(
                        pos.x,
                        pos.y,
                        self.step_size,
                        self.step_size,
                        conf.color.as_mq(),
                    );

                    continue;
                }
                // TL and BR corners
                0b1010 => {
                    // Top triangle
                    draw_triangle(
                        pos.as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );
                    // Bottom triangle
                    draw_triangle(
                        self.local_point(pos, v2!(1.0, 1.0)).as_mq(),
                        self.local_point(pos, conf.lines[1].0).as_mq(),
                        self.local_point(pos, conf.lines[1].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                // TR and BL corners
                0b0101 => {
                    // Top triangle
                    draw_triangle(
                        self.local_point(pos, v2!(1.0, 0.0)).as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );
                    // Bottom triangle
                    draw_triangle(
                        self.local_point(pos, v2!(0.0, 1.0)).as_mq(),
                        self.local_point(pos, conf.lines[1].0).as_mq(),
                        self.local_point(pos, conf.lines[1].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                // Single corner active
                0b1000 => {
                    draw_triangle(
                        pos.as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                0b0100 => {
                    draw_triangle(
                        self.local_point(pos, v2!(1.0, 0.0)).as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                0b0010 => {
                    draw_triangle(
                        self.local_point(pos, v2!(1.0, 1.0)).as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                0b0001 => {
                    draw_triangle(
                        self.local_point(pos, v2!(0.0, 1.0)).as_mq(),
                        self.local_point(pos, conf.lines[0].0).as_mq(),
                        self.local_point(pos, conf.lines[0].1).as_mq(),
                        conf.color.as_mq(),
                    );

                    continue;
                }
                _ => {}
            }

            // Other cases - get the points for these and then triangulate
            let points: &[Vector2<f32>] = match conf.configuration_id {
                // 2 corners active - horizontal or vertical line
                0b1100 => &[
                    v2!(0.0, 0.0),
                    v2!(1.0, 0.0),
                    conf.lines[0].1,
                    conf.lines[0].0,
                ],
                0b0110 => &[
                    v2!(1.0, 0.0),
                    v2!(1.0, 1.0),
                    conf.lines[0].1,
                    conf.lines[0].0,
                ],
                0b0011 => &[
                    v2!(1.0, 1.0),
                    v2!(0.0, 1.0),
                    conf.lines[0].0,
                    conf.lines[0].1,
                ],
                0b1001 => &[
                    v2!(0.0, 1.0),
                    v2!(0.0, 0.0),
                    conf.lines[0].0,
                    conf.lines[0].1,
                ],
                // 3 corners active
                0b0111 => &[
                    conf.lines[0].0,
                    conf.lines[0].1,
                    v2!(1.0, 0.0),
                    v2!(1.0, 1.0),
                    v2!(0.0, 1.0),
                ],
                0b1011 => &[
                    v2!(0.0, 0.0),
                    conf.lines[0].1,
                    conf.lines[0].0,
                    v2!(1.0, 1.0),
                    v2!(0.0, 1.0),
                ],
                // TODO: Maybe try switching the points in line in configuration? for the 2 bellow
                0b1101 => &[
                    v2!(0.0, 0.0),
                    v2!(1.0, 0.0),
                    conf.lines[0].1,
                    conf.lines[0].0,
                    v2!(0.0, 1.0),
                ],
                0b1110 => &[
                    v2!(0.0, 0.0),
                    v2!(1.0, 0.0),
                    v2!(1.0, 1.0),
                    conf.lines[0].1,
                    conf.lines[0].0,
                ],
                _ => {
                    panic!("The renderer does not draw all possible configurations! This is a bug!")
                }
            };

            // Draw the triangulation
            for (a, b, c) in triangulate_convex_polygon(points) {
                draw_triangle(
                    self.local_point(pos, a).as_mq(),
                    self.local_point(pos, b).as_mq(),
                    self.local_point(pos, c).as_mq(),
                    conf.color.as_mq(),
                );
            }
        }
    }
}
