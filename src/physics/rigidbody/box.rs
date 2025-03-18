use crate::math::{v2, Vector2};

use super::{Body, Line, SurfacePoint, Triangle};

pub struct RBox {
    pub center: Vector2<f32>,
    pub width: f32,
    pub height: f32,

    // Chached values for faster computations
    pub(crate) lines: [Line; 4],
    pub(crate) triangulation: [Triangle; 2],
}

impl RBox {
    pub fn new(center: Vector2<f32>, width: f32, height: f32) -> Self {
        // Init with empty cache values, then set them up
        let mut rbox = RBox {
            center,
            width,
            height,
            lines: [
                Line::new(Vector2::zero(), Vector2::zero()),
                Line::new(Vector2::zero(), Vector2::zero()),
                Line::new(Vector2::zero(), Vector2::zero()),
                Line::new(Vector2::zero(), Vector2::zero()),
            ],
            triangulation: [
                Triangle::new(Vector2::zero(), Vector2::zero(), Vector2::zero()),
                Triangle::new(Vector2::zero(), Vector2::zero(), Vector2::zero()),
            ],
        };
        rbox.setup_lines();
        rbox.setup_triangulation();

        rbox
    }

    fn setup_lines(&mut self) {
        let half_width = self.width * 0.5;
        let half_height = self.height * 0.5;

        // TL - TR
        self.lines[0] = Line::new(
            self.center + v2!(-half_width, -half_height),
            self.center + v2!(half_width, -half_height),
        );
        // TR - BR
        self.lines[1] = Line::new(
            self.center + v2!(half_width, -half_height),
            self.center + v2!(half_width, half_height),
        );
        // BR - BL
        self.lines[2] = Line::new(
            self.center + v2!(half_width, half_height),
            self.center + v2!(-half_width, half_height),
        );
        // BL - TL
        self.lines[3] = Line::new(
            self.center + v2!(-half_width, half_height),
            self.center + v2!(-half_width, -half_height),
        );
    }

    /// Lines should be set-up before running this
    fn setup_triangulation(&mut self) {
        let tl = self.lines[0].start;
        let tr = self.lines[1].start;
        let br = self.lines[2].start;
        let bl = self.lines[3].start;

        self.triangulation[0] = Triangle::new(tl, tr, bl);
        self.triangulation[1] = Triangle::new(br, bl, tr);
    }
}

impl Body for RBox {
    /// Find the closest point on the surface of the Box.
    fn closest_surface_point(&self, point: Vector2<f32>) -> SurfacePoint {
        let mut closest_point = self.lines[0].closest_surface_point(point);
        let mut closest_dist_sq = (closest_point.point - point).length_squared();
        for line in &self.lines[1..] {
            let surface_point = line.closest_surface_point(point);
            let dist_sq = (surface_point.point - point).length_squared();
            if dist_sq < closest_dist_sq {
                closest_point = surface_point;
                closest_dist_sq = dist_sq;
            }
        }

        closest_point
    }

    fn is_inside(&self, point: Vector2<f32>) -> bool {
        self.triangulation[0].is_inside(point) || self.triangulation[1].is_inside(point)
    }
}
