use crate::math::{v2, Vector2};

use super::{Body, Line, CollisionInfo};

pub struct TriangleBody {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,

    // Cached values to speed up calculation
    pub(crate) lines: [Line; 3],
    pub(crate) center_of_mass: Vector2<f32>,
}

impl TriangleBody {
    pub fn new(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> TriangleBody {
        TriangleBody {
            a,
            b,
            c,
            lines: [Line::new(a, b), Line::new(b, c), Line::new(c, a)],
            center_of_mass: Self::get_center_of_mass(a, b, c),
        }
    }

    fn get_center_of_mass(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> Vector2<f32> {
        let x = (a.x + b.x + c.x) / 3.0;
        let y = (a.y + b.y + c.y) / 3.0;
        v2!(x, y)
    }

    fn update_cached_values(&mut self) {
        // Setup lines
        self.lines[0] = Line::new(self.a, self.b);
        self.lines[1] = Line::new(self.b, self.c);
        self.lines[2] = Line::new(self.c, self.a);

        // Setup center of mass
        self.center_of_mass = Self::get_center_of_mass(self.a, self.b, self.c);
    }
}

impl Body for TriangleBody {
    fn collision_info(&self, point: Vector2<f32>) -> CollisionInfo {
        let mut closest_point = self.lines[0].collision_info(point);
        let mut closest_dist_sq = (closest_point.point - point).length_squared();
        for line in &self.lines[1..] {
            let surface_point = line.collision_info(point);
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

    fn center_of_mass(&self) -> Vector2<f32> {
        self.center_of_mass
    }
}
