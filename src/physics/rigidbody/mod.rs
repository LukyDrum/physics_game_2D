use crate::math::Vector2;

mod r#box;
mod container;
mod line;
mod triangle;

pub use container::Container;
pub use line::Line;
pub use r#box::RBox;
pub use triangle::Triangle;

pub struct SurfacePoint {
    pub point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
}

/// A physical object that can be simulated in the game world
pub trait Body {
    fn closest_surface_point(&self, point: Vector2<f32>) -> SurfacePoint;

    fn is_inside(&self, point: Vector2<f32>) -> bool;
}
