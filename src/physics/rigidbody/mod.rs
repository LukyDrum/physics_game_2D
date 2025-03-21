use crate::{math::Vector2, utility::runge_kutta};

mod polygon;
mod rb_simulation;

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

pub struct PointCollisionInfo {
    pub surface_point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
}

/// A physical object that can be simulated in the game world
pub trait Body: Send + Sync {
    fn pre_update(&mut self);

    fn state(&self) -> &BodyState;

    fn state_mut(&mut self) -> &mut BodyState;

    fn point_collision_info(&self, point: Vector2<f32>) -> PointCollisionInfo;

    fn contains_point(&self, point: Vector2<f32>) -> bool;

    fn center_of_mass(&self) -> Vector2<f32>;

    fn apply_force_at_point(&mut self, force: Vector2<f32>, point: Vector2<f32>);
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
