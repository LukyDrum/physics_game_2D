use std::collections::LinkedList;

use crate::{game::GameBody, math::Vector2, utility::runge_kutta};

mod polygon;
mod rb_simulation;

use num_traits::{float::FloatCore, real::Real, Float, Signed};
pub use polygon::Polygon;
pub use rb_simulation::RbSimulator;

#[derive(Copy, Clone, PartialEq)]
pub enum BodyBehaviour {
    Dynamic,
    Static,
}

#[derive(Clone)]
pub struct BodyState {
    pub position: Vector2<f32>,
    pub behaviour: BodyBehaviour,
    pub velocity: Vector2<f32>,
    pub mass: f32,
    pub rotation: f32,

    accumulated_force: Vector2<f32>,
}

impl BodyState {
    pub fn new(position: Vector2<f32>, behaviour: BodyBehaviour) -> BodyState {
        BodyState {
            position,
            behaviour,
            velocity: Vector2::zero(),
            mass: 10.0,
            rotation: 0.0,

            accumulated_force: Vector2::zero(),
        }
    }

    pub fn add_force(&mut self, force: Vector2<f32>) {
        self.accumulated_force += force;
    }

    pub fn apply_accumulated_force(&mut self, time_step: f32) {
        let acc = self.accumulated_force / self.mass;
        self.velocity = runge_kutta(self.velocity, time_step, acc);
        self.accumulated_force = Vector2::zero();
    }

    pub fn move_by_velocity(&mut self, time_step: f32) {
        self.position = runge_kutta(self.position, time_step, self.velocity);
    }
}

pub struct PointCollisionData {
    pub surface_point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
}

pub struct BodyProjection {
    min: f32,
    max: f32,
}

impl Default for BodyProjection {
    fn default() -> Self {
        BodyProjection {
            min: f32::MAX,
            max: f32::MIN,
        }
    }
}

impl BodyProjection {
    /// Adds the value into this projection. Handles checking if it is outside the current interval
    /// and updates it.
    pub fn add(&mut self, value: f32) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Returns the size of the overlap between these 2 projections or None if the do not overlap.
    pub fn get_overlap(&self, other: &BodyProjection) -> Option<f32> {
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
}

pub struct BodyCollisionData {
    /// Vector point away from the edge. This will be the normal unit vector of the edge on which
    /// the collision occured.
    pub normal: Vector2<f32>,
    /// The depth of the penetration/collision.
    pub penetration: f32,
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
    fn point_collision_info(&self, point: Vector2<f32>) -> PointCollisionData;

    /// Returns `true` if the point is inside this Body.
    fn contains_point(&self, point: Vector2<f32>) -> bool;

    /// Returns the center of mass of this Body in the global coordinates.
    fn center_of_mass(&self) -> Vector2<f32>;

    /// Returns the projection of this Body onto the provided `axis`.
    fn project_onto_axis(&self, axis: Vector2<f32>) -> BodyProjection;

    /// Returns a list of projection axis from this Body. That is a list of normals of the lines
    /// this body consist of.
    fn projection_axes(&self) -> LinkedList<Vector2<f32>>;

    /// Checks if this Body collides with the `other` Body and if so returns a `BodyCollisionInfo`.
    /// Otherwise returns `None` (meaning they do not collide).
    fn check_collision_against(&self, other: &Box<dyn GameBody>) -> Option<BodyCollisionData>;
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
