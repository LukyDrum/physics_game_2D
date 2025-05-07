use crate::math::{v2, Vector2};

use super::{BodyState, PointCollisionData, PointsProjection};

pub struct CircleInner {
    pub(super) state: BodyState,
    pub radius: f32,
}

impl CircleInner {
    pub(super) fn contains_point(&self, point: Vector2<f32>) -> bool {
        (point - self.state.position).length_squared() <= self.radius.powi(2)
    }

    pub(super) fn point_collision_data(&self, point: Vector2<f32>) -> PointCollisionData {
        let vector = point - self.state.position;
        let normal = vector.normalized();
        let surface_point = self.state.position + normal * self.radius;

        PointCollisionData {
            surface_point,
            surface_normal: normal,
        }
    }

    pub(super) fn calculate_moment_of_inertia(mass: f32, radius: f32) -> f32 {
        0.5 * mass * radius.powi(2)
    }

    pub(super) fn project_onto_axis(&self, axis: Vector2<f32>) -> PointsProjection {
        let mut proj = PointsProjection::default();
        // The direction doesn't matter
        let start = self.state.position - v2!(-1.0, 0.0) * self.radius;
        let end = self.state.position - v2!(1.0, 0.0) * self.radius;

        proj.add(start.dot(axis));
        proj.add(end.dot(axis));

        proj
    }
}
