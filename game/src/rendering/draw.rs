use macroquad::shapes::{draw_line, draw_triangle};

use super::Color;
use crate::{
    physics::rigidbody::RigidBody,
    shapes::{Line, Triangle, Triangulation},
    utility::AsMq,
};

/// Implementors of this trait have the ability to be drawn to the screen.
pub trait Draw {
    fn draw(&self);

    fn draw_with_color(&self, color: Color);
}

const BLACK: Color = Color::rgb(0, 0, 0);

pub fn draw_triangulation(triangulation: &Triangulation, color: Color) {
    for Triangle { a, b, c } in triangulation {
        draw_triangle(a.as_mq(), b.as_mq(), c.as_mq(), color.as_mq());
    }
}

impl Draw for Line {
    fn draw(&self) {
        self.draw_with_color(BLACK);
    }

    fn draw_with_color(&self, color: Color) {
        draw_line(
            self.start.x,
            self.start.y,
            self.end.x,
            self.end.y,
            2.0,
            color.as_mq(),
        );
    }
}

impl Draw for RigidBody {
    fn draw(&self) {
        match self {
            Self::Polygon(inner) => {
                draw_triangulation(inner.global_triangulation(), self.state().color)
            }
        }
    }

    fn draw_with_color(&self, color: Color) {
        match self {
            Self::Polygon(inner) => draw_triangulation(inner.global_triangulation(), color),
        }
    }
}
