use macroquad::math::Vec2;

use crate::{math::Vector2, rendering::Color};

/// A generic trait for converting internal type to MacroQuad equivalent `T`.
pub trait AsMq<T> {
    fn as_mq(&self) -> T;
}

impl AsMq<Vec2> for Vector2<f32> {
    fn as_mq(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl AsMq<macroquad::color::Color> for Color {
    fn as_mq(&self) -> macroquad::color::Color {
        macroquad::color::Color::new(self.r, self.g, self.b, self.a)
    }
}
