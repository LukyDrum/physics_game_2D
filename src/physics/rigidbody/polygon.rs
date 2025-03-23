use std::collections::LinkedList;

use crate::game::GameBody;
use crate::math::{v2, Matrix, Vector2};
use crate::shapes::{triangulate_convex_polygon, Line, Triangulation};

use super::{
    Body, BodyBehaviour, BodyCollisionData, BodyProjection, BodyState, PointCollisionData,
};

/// Simple convex polygon.
pub struct Polygon {
    pub state: BodyState,
    /// These points are the vertices of the polygon - relative to it's position
    pub points: Vec<Vector2<f32>>,

    /// Cached values - they should periodicly update
    pub(super) global_points: Vec<Vector2<f32>>,
    /// Triangulation of the polygon in global space
    pub(super) global_triangulation: Triangulation,
    pub(super) global_lines: Vec<Line>,
}

impl Polygon {
    pub fn new(
        position: Vector2<f32>,
        points: Vec<Vector2<f32>>,
        behaviour: BodyBehaviour,
    ) -> Polygon {
        let points_size = points.len();

        let mut poly = Polygon {
            state: BodyState::new(position, behaviour),
            points,

            global_points: Vec::with_capacity(points_size),
            global_triangulation: Vec::with_capacity(points_size - 2),
            global_lines: Vec::with_capacity(points_size),
        };
        poly.update_inner_values();

        poly
    }

    fn local_point_to_global(&self, point: Vector2<f32>) -> Vector2<f32> {
        let sin = self.state.rotation.sin();
        let cos = self.state.rotation.cos();

        let rot_mat = Matrix::new([[cos, -sin], [sin, cos]]);
        let local = Matrix::new([[point.x], [point.y]]);
        let position = Matrix::new([[self.state.position.x], [self.state.position.y]]);

        let global = rot_mat * local + position;
        let x = *global.get(0, 0);
        let y = *global.get(1, 0);
        v2!(x, y)
    }

    fn update_inner_values(&mut self) {
        // Calculates local points transformed into the global space
        self.global_points.clear();
        for local_point in &self.points {
            self.global_points
                .push(self.local_point_to_global(*local_point));
        }

        // Update global triangulation
        self.global_triangulation = triangulate_convex_polygon(&self.global_points[..]);

        // Update global lines
        self.global_lines.clear();
        let points_size = self.global_points.len();
        for i in 0..points_size {
            self.global_lines.push(Line::new(
                self.global_points[i],
                self.global_points[(i + 1) % points_size],
            ));
        }
    }

    pub fn global_triangulation(&self) -> &Triangulation {
        &self.global_triangulation
    }
}

impl Body for Polygon {
    fn update_inner_values(&mut self) {
        self.update_inner_values();
    }

    fn state(&self) -> &BodyState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut BodyState {
        &mut self.state
    }

    fn point_collision_info(&self, point: Vector2<f32>) -> PointCollisionData {
        let mut closest_line = &self.global_lines[0];
        let mut surface_point = closest_line.closest_point(point);
        let mut dist_sq = (surface_point - point).length_squared();

        for i in 1..self.global_lines.len() {
            let cur_line = &self.global_lines[i];
            let cur_surface_point = cur_line.closest_point(point);
            let cur_dist_sq = (cur_surface_point - point).length_squared();
            if cur_dist_sq < dist_sq {
                closest_line = cur_line;
                surface_point = cur_surface_point;
                dist_sq = cur_dist_sq;
            }
        }

        PointCollisionData {
            surface_point,
            surface_normal: closest_line.normal(),
        }
    }

    fn contains_point(&self, point: Vector2<f32>) -> bool {
        self.global_triangulation
            .iter()
            .any(|trian| trian.contains_point(point))
    }

    fn center_of_mass(&self) -> Vector2<f32> {
        self.global_points
            .iter()
            .fold(Vector2::zero(), |acc, x| acc + *x)
            / self.global_points.len() as f32
    }

    fn project_onto_axis(&self, axis: Vector2<f32>) -> BodyProjection {
        let mut proj = BodyProjection::default();
        for point in &self.global_points {
            let dist = point.dot(axis);
            proj.add(dist);
        }

        proj
    }

    fn projection_axes(&self) -> LinkedList<Vector2<f32>> {
        self.global_lines
            .iter()
            .map(|line| line.normal().abs())
            .collect()
    }

    fn check_collision_against(&self, other: &Box<dyn GameBody>) -> Option<BodyCollisionData> {
        let mut projection_axes = self.projection_axes();
        projection_axes.append(&mut other.projection_axes());

        // Try to project both bodies on each axis
        let mut min_penetration = f32::MAX;
        let mut min_axis = Vector2::zero();
        for axis in projection_axes {
            let proj_a = self.project_onto_axis(axis);
            let proj_b = other.project_onto_axis(axis);

            if let Some(penetration) = proj_a.get_overlap(&proj_b) {
                if penetration < min_penetration {
                    min_penetration = penetration;
                    min_axis = axis;
                }
            } else {
                // If they do not overlap on at least one axis, then they do not collide
                return None;
            }
        }

        Some(BodyCollisionData {
            normal: min_axis,
            penetration: min_penetration,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::game::GameBody;
    use crate::math::{v2, Vector2};
    use crate::physics::rigidbody::{Body, BodyBehaviour, Polygon, Rectangle};

    fn test_poly() -> Polygon {
        Polygon::new(
            v2!(10.0, 10.0),
            vec![v2!(0.0, 5.0), v2!(5.0, 0.0), v2!(-5.0, 0.0)],
            BodyBehaviour::Static,
        )
    }

    #[test]
    fn local_to_global_point() {
        let poly = test_poly();
        let local_point = poly.points[0];

        assert_eq!(local_point, v2!(0.0, 5.0));

        let global_point = poly.local_point_to_global(local_point);
        assert_eq!(global_point, local_point + poly.state.position)
    }

    #[test]
    fn local_to_global_point_rotated() {
        let mut poly = test_poly();
        poly.state.rotation = -PI * 0.5;
        let local_point = poly.points[0];

        assert_eq!(local_point, v2!(0.0, 5.0));

        let global_point = poly.local_point_to_global(local_point);
        assert_eq!(global_point, poly.state.position + v2!(5.0, 0.0))
    }

    #[test]
    fn projection_onto_horizontal_axis() {
        let poly = test_poly();

        // Project onto a horizontal line
        let proj = poly.project_onto_axis(v2!(1.0, 0.0));

        assert_eq!(proj.min, 5.0);
        assert_eq!(proj.max, 15.0);
    }

    #[test]
    fn projection_onto_vertical_axis() {
        let poly = test_poly();

        // Project onto a vertical line
        let proj = poly.project_onto_axis(v2!(0.0, 1.0));

        assert_eq!(proj.min, 10.0);
        assert_eq!(proj.max, 15.0);
    }

    #[test]
    fn rectangles_not_colliding() {
        // Square centered at (0.0, 0.0) with width = 5.0 and height = 5.0
        let rect1 = Box::new(Rectangle!(v2!(0.0, 0.0); 5.0, 5.0; BodyBehaviour::Static));
        // Same square but centered at (0.0, 10.0)
        let rect2: Box<dyn GameBody> =
            Box::new(Rectangle!(v2!(0.0, 10.0); 5.0, 5.0; BodyBehaviour::Static));

        assert!(rect1.check_collision_against(&rect2).is_none())
    }

    #[test]
    fn rectangles_colliding() {
        // Square centered at (0.0, 0.0) with width = 5.0 and height = 5.0
        let rect1 = Box::new(Rectangle!(v2!(0.0, 0.0); 5.0, 5.0; BodyBehaviour::Static));
        // Same square but centered at (0.0, 10.0)
        let rect2: Box<dyn GameBody> =
            Box::new(Rectangle!(v2!(0.0, 4.0); 5.0, 5.0; BodyBehaviour::Static));

        if let Some(collision_data) = rect1.check_collision_against(&rect2) {
            assert_eq!(collision_data.penetration, 1.0)
        } else {
            assert!(false)
        }
    }
}
