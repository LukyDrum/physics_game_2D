use core::f32;
use std::collections::LinkedList;

use crate::math::Vector2;
use crate::shapes::{triangulate_convex_polygon, Line, Triangulation};

use super::{local_point_to_global, BodyState, PointCollisionData, PointsProjection};

pub struct PolygonInner {
    pub(super) state: BodyState,
    /// These points are the vertices of the polygon - relative to it's position
    pub points: Vec<Vector2<f32>>,

    /// Cached values - they should periodicly update
    pub(super) global_points: Vec<Vector2<f32>>,
    /// Triangulation of the polygon in global space
    pub(super) global_triangulation: Triangulation,
    pub(super) global_lines: Vec<Line>,
}

impl PolygonInner {
    pub(super) fn update_inner_values(&mut self) {
        // Calculates local points transformed into the global space
        self.global_points.clear();
        for local_point in &self.points {
            self.global_points
                .push(local_point_to_global(&self.state, *local_point));
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

    /// Returns a normal vector of the provided line that is pointing away from the center of this
    /// polygon.
    fn lines_normal_pointing_outside(&self, line: &Line) -> Vector2<f32> {
        let normal = line.normal();
        let line_to_pos = self.state.position - line.middle();
        // Make the normal point away from this body - away from this center/position
        if normal.dot(line_to_pos) <= 0.0 {
            normal
        } else {
            normal * -1.0
        }
    }

    pub(super) fn point_collision_data(&self, point: Vector2<f32>) -> PointCollisionData {
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
            surface_normal: self.lines_normal_pointing_outside(closest_line),
        }
    }

    pub(super) fn contains_point(&self, point: Vector2<f32>) -> bool {
        self.global_triangulation
            .iter()
            .any(|trian| trian.contains_point(point))
    }

    pub(super) fn center_of_mass(&self) -> Vector2<f32> {
        self.global_points
            .iter()
            .fold(Vector2::zero(), |acc, x| acc + *x)
            / self.global_points.len() as f32
    }

    pub(super) fn project_onto_axis(&self, axis: Vector2<f32>) -> PointsProjection {
        let mut proj = PointsProjection::default();
        for point in &self.global_points {
            let dist = point.dot(axis);
            proj.add(dist);
        }

        proj
    }

    pub(super) fn projection_axes(&self) -> LinkedList<Vector2<f32>> {
        self.global_lines
            .iter()
            .map(|line| self.lines_normal_pointing_outside(line))
            .collect()
    }

    /// `normal` is a normal vector of some line. This vector points away from this body and
    /// represents the direction in which the penetration is minimal. We asume it is normalized.
    /// This function finds a line of this body that has the most similiar normal to `normal`.
    pub(super) fn find_colliding_line(&self, normal: Vector2<f32>) -> Line {
        let mut best_dot = f32::MIN;
        let mut best_line = &self.global_lines[0];
        for line in &self.global_lines {
            let line_normal = self.lines_normal_pointing_outside(line);
            let dot = normal.dot(line_normal);
            if dot > best_dot {
                best_dot = dot;
                best_line = line;
            }
        }

        best_line.clone()
    }

    pub(super) fn calculate_moment_of_inertia(points: &Vec<Vector2<f32>>, mass: f32) -> f32 {
        let mut iter = points.iter().cycle().peekable();
        let mut sum = 0.0;
        let mut sub_sum = 0.0;

        for _ in 0..points.len() {
            // Should be safe to unwrap
            let this = iter.next().unwrap();
            let after = iter.peek().unwrap();

            let a = after.cross(*this);
            let b = this.dot(*this);
            let c = this.dot(**after);
            let d = after.dot(**after);

            sub_sum += a;
            sum += a * (b + c + d);
        }

        mass * (sum / (6.0 * sub_sum)) * 10.0
    }

    pub(super) fn find_contact_points(
        ref_line: Line,
        inc_line: Line,
        seperating_axis: Vector2<f32>,
        ref_body_proj: PointsProjection,
    ) -> Vec<Vector2<f32>> {
        // Projections of the ref_line end points
        let ref_vec = ref_line.vector().normalized();
        let mut ref_proj = PointsProjection::default();
        let mut inc_proj = PointsProjection::default();

        let ref_start_dot = ref_vec.dot(ref_line.start);
        let ref_end_dot = ref_vec.dot(ref_line.end);

        let inc_start_dot = ref_vec.dot(inc_line.start);
        let inc_end_dot = ref_vec.dot(inc_line.end);
        let (inc_start, inc_end) = if inc_start_dot < inc_end_dot {
            (inc_line.start, inc_line.end)
        } else {
            (inc_line.end, inc_line.start)
        };

        ref_proj.add(ref_start_dot);
        ref_proj.add(ref_end_dot);
        inc_proj.add(inc_start_dot);
        inc_proj.add(inc_end_dot);

        // Clipping of lines:
        // Our goal is to get end of `inc_line` if it ends before `ref_line` or the point on `inc_line`
        // that has its projection equal to the projection of the corresponding end of the `ref_line`.
        //
        //      -10      -6        -1        4
        // INC: ##############################
        // REF:          ############

        // Clip incident line to start
        let point_a = if inc_proj.min < ref_proj.min {
            let inc_proj_length = inc_proj.max - inc_proj.min;
            let inc_min_to_ref_min = ref_proj.min - inc_proj.min;
            let proportion = inc_min_to_ref_min / inc_proj_length;

            inc_start + (inc_end - inc_start) * proportion
        } else {
            inc_start
        };

        // Clip incident line to end
        let point_b = if inc_proj.max > ref_proj.max {
            let inc_proj_length = inc_proj.max - inc_proj.min;
            let ref_max_to_inc_max = inc_proj.max - ref_proj.max;
            let proportion = ref_max_to_inc_max / inc_proj_length;

            inc_end + (inc_start - inc_end) * proportion
        } else {
            inc_end
        };

        // Use only points in the correct half - that is inside the reference polygon
        let mut points = Vec::with_capacity(2);
        if ref_body_proj.contains(seperating_axis.dot(point_a)) {
            points.push(point_a);
        }
        if ref_body_proj.contains(seperating_axis.dot(point_b)) {
            points.push(point_b);
        }

        points
    }
}
