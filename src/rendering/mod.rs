mod draw;
mod marching_squares_render;
mod renderer;
mod scalar_field_render;

pub use draw::*;
use macroquad::math::Vec2;
pub use marching_squares_render::MarchingSquaresRenderer;
pub use renderer::Renderer;
pub use scalar_field_render::ScalarFieldRenderer;

use crate::math::Vector2;

#[derive(Default, Clone)]
struct SamplePoint {
    scalar_value: f32,
    color: Color,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn as_mq(&self) -> macroquad::color::Color {
        macroquad::color::Color::new(self.r, self.g, self.b, self.a)
    }
}

pub trait VectorAsMQ {
    fn as_mq(&self) -> Vec2;
}

impl VectorAsMQ for Vector2<f32> {
    fn as_mq(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

pub type Triangle = (Vector2<f32>, Vector2<f32>, Vector2<f32>);

/// Returns triangulation of the convex polygon given by `point`.
/// The points must be in circular order.
pub fn triangulate_convex_polygon(points: &[Vector2<f32>]) -> Vec<Triangle> {
    let count = points.len();
    if count < 3 {
        return Vec::new();
    }

    let ref_point = points[0];
    let mut triangles = Vec::with_capacity(count - 2);

    for i in 2..count {
        triangles.push((ref_point, points[i - 1], points[i]));
    }

    triangles
}
