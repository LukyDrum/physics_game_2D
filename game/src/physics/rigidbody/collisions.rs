use crate::math::Vector2;

use super::{circle::CircleInner, polygon::PolygonInner, BodyCollisionData};

pub fn polygon_polygon_collision(
    this: &PolygonInner,
    other: &PolygonInner,
) -> Option<BodyCollisionData> {
    let other_position = other.state.position;
    let this_position = this.state.position;
    let this_to_other = other_position - this_position;
    let other_to_this = this_position - other_position;

    // Get the possible projection axes and choose only those that point towards the other body
    // (in context of from which body the axis came from).
    let this_projection_axes = this
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

    // Test projection axes of this body - SAT
    for axis in this_projection_axes {
        let proj_a = this.project_onto_axis(axis);
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
        let proj_a = this.project_onto_axis(axis);
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
    let line_a = this.find_colliding_line(min_axis);
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

        ref_body_proj = this.project_onto_axis(min_axis);
    } else {
        ref_line = line_b;
        inc_line = line_a;

        ref_body_proj = other.project_onto_axis(min_axis);
    }

    // Clip the incident line to find the collision points
    let collision_points =
        PolygonInner::find_contact_points(ref_line, inc_line, min_axis, ref_body_proj);

    Some(BodyCollisionData {
        normal: min_axis,
        penetration: min_penetration,
        collision_points,
    })
}

pub fn circle_circle_collision(
    this: &CircleInner,
    other: &CircleInner,
) -> Option<BodyCollisionData> {
    let this_position = this.state.position;
    let other_position = other.state.position;
    let this_to_other = other_position - this_position;

    let radius_sum = this.radius + other.radius;
    let radius_sum_squared = radius_sum.powi(2);

    // Distance of centers is bigger than their summed radiuses -> they do not collide
    if radius_sum_squared < this_to_other.length_squared() {
        return None;
    }

    // Collision normal is the vector from this center to other center
    let normal = this_to_other.normalized();

    // Penetration depth whill be given using the following equality:
    // dist = this.radius + other.radius - penetration
    // => penetration = this.radius + other.radius - dist
    let dist = this_to_other.length();
    let penetration = this.radius + other.radius - dist;

    // The collision point will be the middle point between the edges of the circles along the
    // normal
    let collision_point = this_position + normal * (this.radius - penetration * 0.5);

    Some(BodyCollisionData {
        normal,
        penetration,
        collision_points: vec![collision_point],
    })
}

pub fn polygon_circle_collision(
    polygon: &PolygonInner,
    circle: &CircleInner,
) -> Option<BodyCollisionData> {
    let circle_center = circle.state.position;

    let (mut min_distance_sq, mut min_point, mut normal) =
        (f32::MAX, Vector2::zero(), Vector2::zero());

    for line in &polygon.global_lines {
        let point = line.closest_point(circle_center);
        let dist_sq = (point - circle_center).length_squared();

        if dist_sq < min_distance_sq {
            min_distance_sq = dist_sq;
            min_point = point;
            normal = polygon.lines_normal_pointing_outside(line);
        }
    }

    if min_distance_sq > circle.radius.powi(2) {
        return None;
    }

    let collision_point = min_point;

    // Penetration
    let point_to_center_dist = (collision_point - circle_center).length();
    let penetration = circle.radius - point_to_center_dist;

    Some(BodyCollisionData {
        normal,
        penetration,
        collision_points: vec![collision_point],
    })
}
