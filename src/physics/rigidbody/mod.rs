use crate::math::Vector2;

mod polygon;
mod rb_simulation;

pub use polygon::Polygon;
pub use rb_simulation::RbSimulator;

#[derive(Copy, Clone)]
pub struct BodyState {
    pub position: Vector2<f32>,
    pub velocity: Vector2<f32>,
    pub mass: f32,
    pub rotation: f32,
}

impl BodyState {
    pub fn new(position: Vector2<f32>) -> BodyState {
        BodyState {
            position,
            velocity: Vector2::zero(),
            mass: 10.0,
            rotation: 0.0,
        }
    }
}

pub struct PointCollisionInfo {
    pub surface_point: Vector2<f32>,
    pub surface_normal: Vector2<f32>,
}

/// A physical object that can be simulated in the game world
pub trait Body: Send + Sync {
    fn pre_update(&mut self);

    fn state(&self) -> BodyState;

    fn point_collision_info(&self, point: Vector2<f32>) -> PointCollisionInfo;

    fn contains_point(&self, point: Vector2<f32>) -> bool;

    fn center_of_mass(&self) -> Vector2<f32>;
}

macro_rules! Rectangle {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        let avg_pos: Vector2<f32> = ($a + $b + $c + $d) * 0.25;
        let points = vec![$a - avg_pos, $b - avg_pos, $c - avg_pos, $d - avg_pos];

        Polygon::new(avg_pos, points)
    }};
    ($pos:expr; $width:expr, $height:expr) => {{
        let half_w: f32 = $width * 0.5;
        let half_h: f32 = $height * 0.5;
        let points = vec![
            v2!(-half_w, -half_h),
            v2!(half_w, -half_h),
            v2!(half_w, half_h),
            v2!(-half_w, half_h),
        ];

        Polygon::new($pos, points)
    }};
}

pub(crate) use Rectangle;
