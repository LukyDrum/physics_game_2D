use crate::math::{v2, Matrix, Vector2};
use crate::shapes::{triangulate_convex_polygon, Line, Triangulation};

use super::{Body, BodyBehaviour, BodyState, PointCollisionInfo};

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
    fn pre_update(&mut self) {
        self.update_inner_values();
    }

    fn state(&self) -> &BodyState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut BodyState {
        &mut self.state
    }

    fn point_collision_info(&self, point: Vector2<f32>) -> PointCollisionInfo {
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

        PointCollisionInfo {
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

    fn apply_force_at_point(&mut self, force: Vector2<f32>, point: Vector2<f32>) {
        todo!()
    }
}
