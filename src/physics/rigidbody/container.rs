use crate::math::{v2, Vector2};

use super::{Body, SurfacePoint};

/// Axis-aligned box that keeps stuff INSIDE it.
pub struct Container {
    pub center: Vector2<f32>,
    pub half_width: f32,
    pub half_height: f32,
}

impl Container {
    pub fn new(center: Vector2<f32>, width: f32, height: f32) -> Self {
        Container {
            center,
            half_width: width * 0.5,
            half_height: height * 0.5,
        }
    }
}

const MAX_OFFSET: f32 = 5.0;

impl Body for Container {
    fn closest_surface_point(&self, point: Vector2<f32>) -> SurfacePoint {
        let mut x = point.x.clamp(
            self.center.x - self.half_width,
            self.center.x + self.half_width,
        );
        let mut y = point.y.clamp(
            self.center.y - self.half_height,
            self.center.y + self.half_height,
        );

        // If `x` did not change than `y` must have changed, or it doesnt matter what we return.
        let surface_normal = if x != point.x {
            v2!(1.0, 0.0)
        } else {
            v2!(0.0, 1.0)
        };

        // *HACKS*: 🙈
        // This should help particles to NOT group in corners
        if x != point.x && y != point.y {
            let x_diff = x - point.x;
            let y_diff = y - point.y;
            if x_diff.abs() > y_diff.abs() {
                x += x_diff.clamp(-MAX_OFFSET, MAX_OFFSET);
            } else {
                y += y_diff.clamp(-MAX_OFFSET, MAX_OFFSET);
            }
        }

        SurfacePoint {
            point: v2!(x, y),
            surface_normal,
        }
    }

    /// The `container` defines space that is **outside**. Everything else is inside.
    fn is_inside(&self, point: Vector2<f32>) -> bool {
        let diff = (self.center - point).abs();
        diff.x > self.half_width || diff.y > self.half_height
    }
}
