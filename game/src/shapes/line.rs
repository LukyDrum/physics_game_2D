use crate::math::Vector2;

#[derive(Clone)]
pub struct Line {
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
    vector: Vector2<f32>,
    unit_normal: Vector2<f32>,
}

impl Line {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>) -> Line {
        let vector = end - start;
        let unit_normal = vector.normalized().normal();
        Line {
            start,
            end,
            vector,
            unit_normal,
        }
    }

    pub fn closest_point(&self, point: Vector2<f32>) -> Vector2<f32> {
        let start_to_point = point - self.start;
        let dot = self.vector.dot(start_to_point);
        let t = (dot / self.vector.length_squared()).clamp(0.0, 1.0);

        self.start + self.vector * t
    }

    pub fn normal(&self) -> Vector2<f32> {
        self.unit_normal
    }

    pub fn vector(&self) -> Vector2<f32> {
        self.vector
    }

    pub fn middle(&self) -> Vector2<f32> {
        (self.start + self.end) * 0.5
    }
}
