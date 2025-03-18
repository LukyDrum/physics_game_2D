use crate::math::{v2, Vector2};

use super::{Body, Line, SurfacePoint};

pub struct Triangle {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,

    // Cached values to speed up calculation
    pub(crate) lines: [Line; 3],
}

impl Triangle {
    pub fn new(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> Triangle {
        Triangle {
            a,
            b,
            c,
            lines: [Line::new(a, b), Line::new(b, c), Line::new(c, a)],
        }
    }

    fn setup_lines(&mut self) {
        self.lines[0] = Line::new(self.a, self.b);
        self.lines[1] = Line::new(self.b, self.c);
        self.lines[2] = Line::new(self.c, self.a);
    }
}

impl Body for Triangle {
    fn closest_surface_point(&self, point: Vector2<f32>) -> SurfacePoint {
        let mut closest_point = self.lines[0].closest_surface_point(point);
        let mut closest_dist_sq = (closest_point.point - point).length_squared();
        for line in &self.lines[1..] {
            let surface_point = line.closest_surface_point(point);
            let dist_sq = (surface_point.point - point).length_squared();
            if dist_sq < closest_dist_sq {
                closest_point = surface_point;
                closest_dist_sq = dist_sq;
            }
        }

        closest_point
    }

    /// Calculates whether the point is inside using barycentric coordinates
    fn is_inside(&self, point: Vector2<f32>) -> bool {
        let v0 = self.b - self.a;
        let v1 = self.c - self.a;
        let v2 = point - self.a;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        v >= 0.0 && w >= 0.0 && u >= 0.0
    }
}
