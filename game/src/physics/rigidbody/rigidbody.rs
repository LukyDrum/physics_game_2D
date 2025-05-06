use crate::math::Vector2;

use super::{
    collisions::polygon_polygon_collision, polygon::PolygonInner, BodyBehaviour, BodyCollisionData,
    BodyState, PointCollisionData,
};

pub enum RigidBody {
    Polygon(PolygonInner),
}

impl RigidBody {
    pub fn check_collision(first: &RigidBody, second: &RigidBody) -> Option<BodyCollisionData> {
        match (first, second) {
            (Self::Polygon(first), Self::Polygon(second)) => {
                polygon_polygon_collision(first, second)
            }
        }
    }

    pub fn new_polygon(
        position: Vector2<f32>,
        points: Vec<Vector2<f32>>,
        behaviour: BodyBehaviour,
    ) -> RigidBody {
        let points_size = points.len();
        let state = BodyState::new(position, 10.0, behaviour);

        let mut poly = PolygonInner {
            state,
            points,
            global_points: Vec::with_capacity(points_size),
            global_triangulation: Vec::with_capacity(points_size - 2),
            global_lines: Vec::with_capacity(points_size),
        };
        poly.update_inner_values();

        // Calculate moment of inertia
        poly.state.moment_of_inertia =
            PolygonInner::calculate_moment_of_inertia(&poly.points, poly.state.mass);

        RigidBody::Polygon(poly)
    }

    pub fn state(&self) -> &BodyState {
        match self {
            Self::Polygon(inner) => &inner.state,
        }
    }

    pub fn state_mut(&mut self) -> &mut BodyState {
        match self {
            Self::Polygon(inner) => &mut inner.state,
        }
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        match self {
            // Polygon requires an update of inner state after changing position
            Self::Polygon(inner) => {
                inner.state.position = position;
                inner.update_inner_values();
            }
        }
    }

    pub fn contains_point(&self, point: Vector2<f32>) -> bool {
        match self {
            Self::Polygon(inner) => inner.contains_point(point),
        }
    }

    pub fn update_inner_values(&mut self) {
        match self {
            Self::Polygon(inner) => inner.update_inner_values(),
        }
    }

    pub fn center_of_mass(&self) -> Vector2<f32> {
        match self {
            Self::Polygon(inner) => inner.center_of_mass(),
        }
    }

    pub fn point_collision_data(&self, point: Vector2<f32>) -> PointCollisionData {
        match self {
            Self::Polygon(inner) => inner.point_collision_data(point),
        }
    }
}
