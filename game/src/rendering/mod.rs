mod draw;
mod marching_squares_render;
mod renderer;

use serde_derive::{Deserialize, Serialize};

pub use draw::*;
pub use marching_squares_render::MarchingSquaresRenderer;
pub use renderer::Renderer;

#[derive(Default, Clone)]
struct SamplePoint {
    scalar_value: f32,
    color: Color,
}

/// Representation of a RGBA color.
/// Acts only as a container for the 4 values.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a new RGBA color from these floats where each should be in [0..1] range.
    /// Other values will be clamped into that range.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
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
}
