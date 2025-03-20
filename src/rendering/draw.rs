use macroquad::shapes::{draw_line, draw_triangle};

use super::{Color, VectorAsMQ};
use crate::physics::rigidbody::{Line, BoxBody, TriangleBody};

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

impl Draw for BoxBody {
    fn draw(&self) {
        for trian in &self.triangulation {
            trian.draw();
        }
    }
}

impl Draw for TriangleBody {
    fn draw(&self) {
        draw_triangle(
            self.a.as_mq(),
            self.b.as_mq(),
            self.c.as_mq(),
            BLACK.as_mq(),
        );
    }
}
