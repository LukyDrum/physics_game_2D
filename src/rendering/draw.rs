use macroquad::{
    math::Vec2,
    shapes::{draw_line, draw_rectangle_lines, draw_triangle},
};

use super::Color;
use crate::{
    math::Vector2,
    physics::rigidbody::{Container, Line, RBox, Triangle},
};

fn vector2_as_mq(vector: Vector2<f32>) -> Vec2 {
    Vec2::new(vector.x, vector.y)
}

/// Implementors of this trait have the ability to be drawn to the screen.
pub trait Draw {
    fn draw(&self);
}

const BLACK: Color = Color::rgb(0, 0, 0);

impl Draw for Line {
    fn draw(&self) {
        let a = self.start;
        let b = self.end;
        draw_line(a.x, a.y, b.x, b.y, 2.0, BLACK.as_mq());
    }
}

impl Draw for RBox {
    fn draw(&self) {
        for trian in &self.triangulation {
            trian.draw();
        }
    }
}

impl Draw for Triangle {
    fn draw(&self) {
        draw_triangle(
            vector2_as_mq(self.a),
            vector2_as_mq(self.b),
            vector2_as_mq(self.c),
            BLACK.as_mq(),
        );
    }
}

impl Draw for Container {
    fn draw(&self) {
        draw_rectangle_lines(
            self.center.x - self.half_width,
            self.center.y - self.half_height,
            self.half_width * 2.0,
            self.half_height * 2.0,
            2.0,
            BLACK.as_mq(),
        );
    }
}
