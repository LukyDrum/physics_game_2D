use crate::math::{v2, Vector2};

use super::{Body, CollisionInfo};

pub struct Line {
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
    // Commonly used values - cached to save some calculations
    vector: Vector2<f32>,
    unit_normal: Vector2<f32>,
}

impl Line {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>) -> Self {
        let vector = end - start;
        Line {
            start,
            end,
            vector,
            unit_normal: vector.normal().normalized(),
        }
    }
}

impl Body for Line {
    fn collision_info(&self, point: Vector2<f32>) -> CollisionInfo {
        let start_to_point = point - self.start;
        let dot = self.vector.dot(start_to_point);
        let t = (dot / self.vector.length_squared()).clamp(0.0, 1.0);

        let point = self.start + self.vector * t;
        CollisionInfo {
            point,
            surface_normal: self.unit_normal,
        }
    }

    /// Always false as a line does not have any inside
    fn is_inside(&self, _point: Vector2<f32>) -> bool {
        false
    }

    fn center_of_mass(&self) -> Vector2<f32> {
        v2!((self.start.x + self.end.x) / 2.0, (self.start.y + self.end.y) / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        math::Vector2,
        physics::rigidbody::{Body, Line},
    };

    #[test]
    fn closest_point_on_line() {
        let line = Line::new(Vector2::new(0.0, 0.0), Vector2::new(5.0, 0.0));
        let point = Vector2::new(3.0, 4.0);

        assert_eq!(
            line.collision_info(point).point,
            Vector2::new(3.0, 0.0)
        )
    }
}
