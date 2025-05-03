use core::f32;
use serde_derive::{Deserialize, Serialize};
use std::collections::LinkedList;

use crate::{game::GameBody, math::Vector2, rendering::Color, shapes::Line, utility::runge_kutta};

mod polygon;
mod rb_simulation;

use num_traits::Zero;
pub use polygon::Polygon;
pub use rb_simulation::RbSimulator;

// Base values for body state properties
const DEFAULT_ELASTICITY: f32 = 0.4;
const DEFAULT_STATIC_FRICTION: f32 = 0.3;
const DEFAULT_DYNAMIC_FRICTION: f32 = 0.2;

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
    pub elasticity: f32,
    /// A value between 0 and 1. Describes the friction between 2 stationary bodies.
    pub static_friction: f32,
    /// The dynamic friction coefficient of this body. A value between 0 and 1.
    pub dynamic_friction: f32,

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
            elasticity: DEFAULT_ELASTICITY,
            static_friction: DEFAULT_STATIC_FRICTION,
            dynamic_friction: DEFAULT_DYNAMIC_FRICTION,

            color: Color::rgb(0, 0, 0),

            accumulated_force: Vector2::zero(),
            accumulated_torque: 0.0,
        }
    }

    pub fn set_mass(&mut self, new_mass: f32) {
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

pub struct PointCollisionData {
    pub surface_point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
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

/// A physical object that can be simulated in the game world
pub trait Body: Send + Sync {
    /// Updates inner stored values such as global points or lines.
    fn update_inner_values(&mut self);

    fn state(&self) -> &BodyState;

    fn state_mut(&mut self) -> &mut BodyState;

    /// Returns a collision info about the collision of this Body and a point.
    /// The returned info will only properly make sense if the point is inside the Body. That can
    /// be checked using `Self::contains_point`.
    fn point_collision_data(&self, point: Vector2<f32>) -> PointCollisionData;

    /// Returns `true` if the point is inside this Body.
    fn contains_point(&self, point: Vector2<f32>) -> bool;

    /// Returns the center of mass of this Body in the global coordinates.
    fn center_of_mass(&self) -> Vector2<f32>;

    /// Returns the projection of this Body onto the provided `axis`.
    fn project_onto_axis(&self, axis: Vector2<f32>) -> PointsProjection;

    /// Returns a list of projection axis from this Body. That is a list of normals of the lines
    /// this body consist of. They will always be pointing away from the body.
    fn projection_axes(&self) -> LinkedList<Vector2<f32>>;

    /// Returns the colliding line based on the collision normal.
    fn find_colliding_line(&self, normal: Vector2<f32>) -> Line;

    /// Checks if this Body collides with the `other` Body and if so returns a `BodyCollisionInfo`.
    /// Otherwise returns `None` (meaning they do not collide).
    fn check_collision_against(&self, other: &Box<dyn GameBody>) -> Option<BodyCollisionData>;

    fn set_position(&mut self, new_position: Vector2<f32>) {
        self.state_mut().position = new_position;
        self.update_inner_values();
    }
}

macro_rules! Rectangle {
    ($a:expr, $b:expr, $c:expr, $d:expr; $behaviour:expr) => {{
        let avg_pos: Vector2<f32> = ($a + $b + $c + $d) * 0.25;
        let points = vec![$a - avg_pos, $b - avg_pos, $c - avg_pos, $d - avg_pos];

        Polygon::new(avg_pos, points, $behaviour)
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

        Polygon::new($pos, points, $behaviour)
    }};
}

pub(crate) use Rectangle;
