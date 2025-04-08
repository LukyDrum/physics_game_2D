use crate::math::Vector2;

pub type Triangulation = Vec<Triangle>;

pub fn triangulate_convex_polygon(points: &[Vector2<f32>]) -> Triangulation {
    let count = points.len();
    if count < 3 {
        return Vec::new();
    }

    let ref_point = points[0];
    let mut triangles = Vec::with_capacity(count - 2);

    for i in 2..count {
        triangles.push(Triangle {
            a: ref_point,
            b: points[i - 1],
            c: points[i],
        });
    }

    triangles
}

pub struct Triangle {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,
}

impl Triangle {
    /// Calculates whether the point is inside using barycentric coordinates
    pub fn contains_point(&self, point: Vector2<f32>) -> bool {
        let v0 = self.b - self.a;
        let v1 = self.c - self.a;
        let v2 = point - self.a;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        v >= 0.0 && w >= 0.0 && u >= 0.0
    }
}
