mod marching_squares_render;
mod renderer;
mod scalar_field_render;

pub use marching_squares_render::MarchingSquaresRenderer;
pub use renderer::Renderer;
pub use scalar_field_render::ScalarFieldRenderer;

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
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub fn as_mq(&self) -> macroquad::color::Color {
        macroquad::color::Color::new(self.r, self.g, self.b, self.a)
    }
}
