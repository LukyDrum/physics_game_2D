use crate::math::Vector2;

mod r#box;
mod container;
mod line;
mod triangle;

pub use container::Container;
pub use line::Line;
pub use r#box::RBox;
pub use triangle::Triangle;

pub struct CollisionInfo {
    pub point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
}

/// A physical object that can be simulated in the game world
pub trait Body {
    fn collision_info(&self, point: Vector2<f32>) -> CollisionInfo;

    fn is_inside(&self, point: Vector2<f32>) -> bool;

    fn center_of_mass(&self) -> Vector2<f32>;
}
