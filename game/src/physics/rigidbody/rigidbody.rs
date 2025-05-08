use crate::math::Vector2;

use super::{
    circle::CircleInner,
    collisions::{circle_circle_collision, polygon_circle_collision, polygon_polygon_collision},
    polygon::PolygonInner,
    BodyBehaviour, BodyCollisionData, BodyState,
};

pub enum RigidBody {
    Polygon(PolygonInner),
    Circle(CircleInner),
}

impl RigidBody {
    pub fn check_collision(first: &RigidBody, second: &RigidBody) -> Option<BodyCollisionData> {
        match (first, second) {
            // Polygon - Polygon
            (Self::Polygon(first), Self::Polygon(second)) => {
                polygon_polygon_collision(first, second)
            }
            // Circle - Circle
            (Self::Circle(first), Self::Circle(second)) => circle_circle_collision(first, second),
            // Polygon - Circle / Circle - Polygon
            (Self::Polygon(polygon), Self::Circle(circle)) => {
                polygon_circle_collision(polygon, circle)
            }
            (Self::Circle(circle), Self::Polygon(polygon)) => {
                let mut data = polygon_circle_collision(polygon, circle);
                // Flip the sign of the normal
                if let Some(data) = &mut data {
                    data.normal *= -1.0;
                }
                data
            }
        }
    }

    pub fn new_polygon(
        position: Vector2<f32>,
        points: Vec<Vector2<f32>>,
        behaviour: BodyBehaviour,
    ) -> RigidBody {
        let points_size = points.len();
        let state = BodyState::new(position, 1_000.0, behaviour);

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

    pub fn new_circle(position: Vector2<f32>, radius: f32, behaviour: BodyBehaviour) -> RigidBody {
        let mut state = BodyState::new(position, 1_000.0, behaviour);
        state.moment_of_inertia = CircleInner::calculate_moment_of_inertia(state.mass, radius);

        let circle = CircleInner { state, radius };

        RigidBody::Circle(circle)
    }

    pub fn state(&self) -> &BodyState {
        match self {
            Self::Polygon(inner) => &inner.state,
            Self::Circle(inner) => &inner.state,
        }
    }

    pub fn state_mut(&mut self) -> &mut BodyState {
        match self {
            Self::Polygon(inner) => &mut inner.state,
            Self::Circle(inner) => &mut inner.state,
        }
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        match self {
            // Polygon requires an update of inner state after changing position
            Self::Polygon(inner) => {
                inner.state.position = position;
                inner.update_inner_values();
            }
            Self::Circle(inner) => inner.state.position = position,
        }
    }

    pub fn contains_point(&self, point: Vector2<f32>) -> bool {
        match self {
            Self::Polygon(inner) => inner.contains_point(point),
            Self::Circle(inner) => inner.contains_point(point),
        }
    }

    pub fn update_inner_values(&mut self) {
        match self {
            Self::Polygon(inner) => inner.update_inner_values(),
            Self::Circle(_) => {}
        }
    }

    pub fn center_of_mass(&self) -> Vector2<f32> {
        match self {
            Self::Polygon(inner) => inner.center_of_mass(),
            Self::Circle(inner) => inner.state.position,
        }
    }
}
