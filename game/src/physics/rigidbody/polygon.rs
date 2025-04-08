use core::f32;
use std::collections::LinkedList;

use crate::game::GameBody;
use crate::math::{v2, Matrix, Vector2};
use crate::shapes::{triangulate_convex_polygon, Line, Triangulation};

use super::{
    Body, BodyBehaviour, BodyCollisionData, BodyState, PointCollisionData, PointsProjection,
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
            state: BodyState::new(position, 10.0, behaviour),
            points,

            global_points: Vec::with_capacity(points_size),
            global_triangulation: Vec::with_capacity(points_size - 2),
            global_lines: Vec::with_capacity(points_size),
        };
        poly.update_inner_values();

        // Calculate moment of inertia
        poly.state.moment_of_inertia = calculate_moment_of_inertia(&poly.points, poly.state.mass);

        poly
    }

    fn local_point_to_global(&self, point: Vector2<f32>) -> Vector2<f32> {
        let sin = self.state.orientation.sin();
        let cos = self.state.orientation.cos();

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

    fn point_collision_data(&self, point: Vector2<f32>) -> PointCollisionData {
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

    fn project_onto_axis(&self, axis: Vector2<f32>) -> PointsProjection {
        let mut proj = PointsProjection::default();
        for point in &self.global_points {
            let dist = point.dot(axis);
            proj.add(dist);
        }

        proj
    }

    fn projection_axes(&self) -> LinkedList<Vector2<f32>> {
        self.global_lines
            .iter()
            .map(|line| self.lines_normal_pointing_outside(line))
            .collect()
    }

    /// `normal` is a normal vector of some line. This vector points away from this body and
    /// represents the direction in which the penetration is minimal. We asume it is normalized.
    /// This function finds a line of this body that has the most similiar normal to `normal`.
    fn find_colliding_line(&self, normal: Vector2<f32>) -> Line {
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

    fn check_collision_against(&self, other: &Box<dyn GameBody>) -> Option<BodyCollisionData> {
        let other_position = other.state().position;
        let this_position = self.state().position;
        let this_to_other = other_position - this_position;
        let other_to_this = this_position - other_position;

        // Get the possible projection axes and choose only those that point towards the other body
        // (in context of from which body the axis came from).
        let this_projection_axes = self
            .projection_axes()
            .into_iter()
            .filter(|axis| axis.dot(this_to_other) >= 0.0);
        let other_projection_axes = other
            .projection_axes()
            .into_iter()
            .filter(|axis| axis.dot(other_to_this) >= 0.0);

        // Try to project both bodies on each axis
        let mut min_penetration = f32::MAX;
        let mut min_axis = Vector2::zero();

        // Test projection axes of this body
        for axis in this_projection_axes {
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
        // Test projection axes of the other body
        let mut axis_is_from_other = false;
        for axis in other_projection_axes {
            let proj_a = self.project_onto_axis(axis);
            let proj_b = other.project_onto_axis(axis);

            if let Some(penetration) = proj_a.get_overlap(&proj_b) {
                if penetration < min_penetration {
                    min_penetration = penetration;
                    min_axis = axis;
                    axis_is_from_other = true;
                }
            } else {
                // If they do not overlap on at least one axis, then they do not collide
                return None;
            }
        }

        if axis_is_from_other {
            min_axis *= -1.0;
        }

        // Find collision manifold points
        // Find the "best" lines from this body and the other
        let line_a = self.find_colliding_line(min_axis);
        // Negate `min_axis` so that it points away from the body
        let line_b = other.find_colliding_line(min_axis * -1.0);

        // Find the reference and incident line
        let (ref_line, inc_line);
        let ref_body_proj;
        // Select the line that is more perpendicular to the normal
        if line_a.vector().normalized().dot(min_axis).abs()
            <= line_b.vector().normalized().dot(min_axis).abs()
        {
            ref_line = line_a;
            inc_line = line_b;

            ref_body_proj = self.project_onto_axis(min_axis);
        } else {
            ref_line = line_b;
            inc_line = line_a;

            ref_body_proj = other.project_onto_axis(min_axis);
        }

        // Clip the incident line to find the collision points
        let collision_points = find_contact_points(ref_line, inc_line, min_axis, ref_body_proj);

        Some(BodyCollisionData {
            normal: min_axis,
            penetration: min_penetration,
            collision_points,
        })
    }
}

fn calculate_moment_of_inertia(points: &Vec<Vector2<f32>>, mass: f32) -> f32 {
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

fn find_contact_points(
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

    fn test_rect() -> Polygon {
        Rectangle!(v2!(0.0, 0.0); 5.0, 5.0; BodyBehaviour::Static)
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
        poly.state.orientation = -PI * 0.5;
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
    fn outside_pointing_line_normal() {
        let rect = test_rect();
        // This should be the top line
        let line_top = &rect.global_lines[0];
        let line_right = &rect.global_lines[1];
        let line_bottom = &rect.global_lines[2];
        let line_left = &rect.global_lines[3];

        let line_top_normal = rect.lines_normal_pointing_outside(line_top);
        let line_right_normal = rect.lines_normal_pointing_outside(line_right);
        let line_bottom_normal = rect.lines_normal_pointing_outside(line_bottom);
        let line_left_normal = rect.lines_normal_pointing_outside(line_left);

        assert_eq!(line_top_normal, v2!(0.0, -1.0));
        assert_eq!(line_right_normal, v2!(1.0, 0.0));
        assert_eq!(line_bottom_normal, v2!(0.0, 1.0));
        assert_eq!(line_left_normal, v2!(-1.0, 0.0));
    }

    #[test]
    fn rectangles_not_colliding() {
        // Square centered at (0.0, 0.0) with width = 5.0 and height = 5.0
        let rect1 = Box::new(test_rect());
        // Same square but centered at (0.0, 100.0)
        let mut rect2: Box<dyn GameBody> = Box::new(test_rect());
        rect2.state_mut().position = v2!(100.0, 100.0);
        rect2.update_inner_values();

        assert!(rect1.check_collision_against(&rect2).is_none())
    }

    #[test]
    fn rectangles_colliding() {
        // Square centered at (0.0, 0.0) with width = 5.0 and height = 5.0
        let rect1 = Box::new(test_rect());
        // Same square but centered at (0.0, 4.0)
        let mut rect2: Box<dyn GameBody> = Box::new(test_rect());
        rect2.state_mut().position = v2!(0.0, 4.0);
        rect2.update_inner_values();

        if let Some(collision_data) = rect1.check_collision_against(&rect2) {
            assert_eq!(collision_data.penetration, 1.0)
        } else {
            assert!(false)
        }
    }
}
