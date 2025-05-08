use core::f32;
use serde_derive::{Deserialize, Serialize};

use crate::{
    math::{v2, Matrix, Vector2},
    rendering::Color,
    utility::runge_kutta,
};

mod circle;
mod collisions;
mod polygon;
mod rb_simulation;
mod rigidbody;

use num_traits::Zero;
pub use rb_simulation::{RbSimulator, SharedProperty, SharedPropertySelection};
pub use rigidbody::RigidBody;

// Base values for body state properties
pub const DEFAULT_ELASTICITY: f32 = 0.4;
pub const DEFAULT_STATIC_FRICTION: f32 = 0.3;
pub const DEFAULT_DYNAMIC_FRICTION: f32 = 0.2;

/// Describes how does the Body behave in the simulation:
///   - `Dynamic` is a body that is affected by gravity and other forces and collides with other bodies.
///   - `Static` is a body that is not affected by forces, but still collides with other bodies
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodyBehaviour {
    Dynamic,
    Static,
}

impl Default for BodyBehaviour {
    fn default() -> Self {
        Self::Dynamic
    }
}

pub struct BodyForceAccumulation {
    pub force: Vector2<f32>,
    pub torque: f32,
}

impl BodyForceAccumulation {
    pub fn empty() -> Self {
        BodyForceAccumulation {
            force: Vector2::zero(),
            torque: 0.0,
        }
    }

    /// Applies the force to the body affecting both linear force as well as torque.
    /// The `radius` is the vector from the center of mass of the object to the collision point.
    pub fn add_force_at_radius(&mut self, force: Vector2<f32>, radius: Vector2<f32>) {
        self.force += force;
        self.torque += radius.cross(force);
    }
}

/// Contains values that are universal for any Body regardless of it being a polygon or a circle
/// (or someting else).
#[derive(Clone, Default)]
pub struct BodyState {
    // BASIC VALUES for 2D space
    pub position: Vector2<f32>,
    /// Linear velocity measured in pixels per second (1 pixel = 1 cm ingame)
    pub velocity: Vector2<f32>,
    /// Angular velocity measured in radians
    pub angular_velocity: f32,
    /// Rotation of the body measured in radians
    pub orientation: f32,

    // PROPERTIES
    pub behaviour: BodyBehaviour,
    pub(crate) mass: f32,
    pub(crate) moment_of_inertia: f32,
    /// The restitution coefficient, aka coefficient of elasticity, aka bounciness.
    /// A value between 0 (no bounce) and 1 (100% bounce).
    pub elasticity: SharedProperty<f32>,
    /// A value between 0 and 1. Describes the friction between 2 stationary bodies.
    pub static_friction: SharedProperty<f32>,
    /// The dynamic friction coefficient of this body. A value between 0 and 1.
    pub dynamic_friction: SharedProperty<f32>,

    // OTHER PROPERTIES
    pub color: Color,

    // ACCUMULATED FORCES waiting to be applied
    pub(crate) accumulated_force: Vector2<f32>,
    pub(crate) accumulated_torque: f32,
}

impl BodyState {
    pub fn new(position: Vector2<f32>, mass: f32, behaviour: BodyBehaviour) -> BodyState {
        BodyState {
            position,
            velocity: Vector2::zero(),
            angular_velocity: 0.0,
            orientation: 0.0,

            behaviour,
            mass,
            // Set it to mass just so it is not empty - it will be set by the body when it is
            // created
            moment_of_inertia: mass,
            elasticity: SharedProperty::Value(DEFAULT_ELASTICITY),
            static_friction: SharedProperty::Value(DEFAULT_STATIC_FRICTION),
            dynamic_friction: SharedProperty::Value(DEFAULT_DYNAMIC_FRICTION),
            color: Color::rgb(0, 0, 0),

            accumulated_force: Vector2::zero(),
            accumulated_torque: 0.0,
        }
    }

    pub fn set_mass(&mut self, new_mass: f32) {
        // Should generaly work
        self.moment_of_inertia = (self.moment_of_inertia / self.mass) * new_mass;
        self.mass = new_mass;
    }

    pub fn mass(&self) -> f32 {
        if self.behaviour == BodyBehaviour::Static {
            f32::INFINITY
        } else {
            self.mass
        }
    }

    pub fn moment_of_inertia(&self) -> f32 {
        if self.behaviour == BodyBehaviour::Static {
            f32::INFINITY
        } else {
            self.moment_of_inertia
        }
    }

    pub fn add_force(&mut self, force: Vector2<f32>) {
        self.accumulated_force += force;
    }

    pub fn add_force_accumulation(&mut self, force_accumulation: BodyForceAccumulation) {
        self.accumulated_force += force_accumulation.force;
        self.accumulated_torque += force_accumulation.torque;
    }

    pub fn apply_accumulated_forces(&mut self, time_step: f32) {
        if !self.accumulated_force.is_zero() {
            let acc = self.accumulated_force / self.mass;
            self.velocity = runge_kutta(self.velocity, time_step, acc);
            self.accumulated_force = Vector2::zero();
        }

        if !self.accumulated_torque.is_zero() {
            let angular_acc = self.accumulated_torque / self.moment_of_inertia;
            self.angular_velocity = runge_kutta(self.angular_velocity, time_step, angular_acc);
            self.accumulated_torque = 0.0;
        }
    }

    pub fn move_by_velocity(&mut self, time_step: f32) {
        self.position = runge_kutta(self.position, time_step, self.velocity);
        self.orientation = runge_kutta(self.orientation, time_step, self.angular_velocity);
    }
}

#[derive(Debug)]
pub struct PointsProjection {
    min: f32,
    max: f32,
}

impl Default for PointsProjection {
    fn default() -> Self {
        PointsProjection {
            min: f32::MAX,
            max: f32::MIN,
        }
    }
}

impl PointsProjection {
    /// Adds the value into this projection. Handles checking if it is outside the current interval
    /// and updates it.
    pub fn add(&mut self, value: f32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Returns the size of the overlap between these 2 projections or None if the do not overlap.
    pub fn get_overlap(&self, other: &PointsProjection) -> Option<f32> {
        if self.max > other.min && other.max > self.min {
            Some(
                (self.min - other.max)
                    .abs()
                    .min((self.max - other.min).abs()),
            )
        } else {
            None
        }
    }

    pub fn contains(&self, value: f32) -> bool {
        value >= self.min && value <= self.max
    }
}

#[derive(Clone)]
pub struct BodyCollisionData {
    /// Vector point away from the edge. This will be the normal unit vector of the edge on which
    /// the collision occured.
    pub normal: Vector2<f32>,
    /// The depth of the penetration/collision.
    pub penetration: f32,
    /// Points of the collision manifold. There should be 1 or 2 points
    pub collision_points: Vec<Vector2<f32>>,
}

macro_rules! Rectangle {
    ($a:expr, $b:expr, $c:expr, $d:expr; $behaviour:expr) => {{
        let avg_pos: Vector2<f32> = ($a + $b + $c + $d) * 0.25;
        let points = vec![$a - avg_pos, $b - avg_pos, $c - avg_pos, $d - avg_pos];

        RigidBody::new_polygon(avg_pos, points, $behaviour)
    }};
    ($pos:expr; $width:expr, $height:expr; $behaviour:expr) => {{
        let half_w: f32 = $width * 0.5;
        let half_h: f32 = $height * 0.5;
        let points = vec![
            v2!(-half_w, -half_h),
            v2!(half_w, -half_h),
            v2!(half_w, half_h),
            v2!(-half_w, half_h),
        ];

        RigidBody::new_polygon($pos, points, $behaviour)
    }};
}

pub(crate) use Rectangle;

fn local_point_to_global(state: &BodyState, point: Vector2<f32>) -> Vector2<f32> {
    let rot_mat = Matrix::rotation_matrix(state.orientation);
    let local = Matrix::from(point);
    let position = Matrix::from(state.position);

    let global = rot_mat * local + position;
    let x = *global.get(0, 0);
    let y = *global.get(1, 0);
    v2!(x, y)
}
