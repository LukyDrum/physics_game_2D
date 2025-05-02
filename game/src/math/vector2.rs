use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use serde::{Serialize, Deserialize};
use num_traits::{Float, Num, NumAssignOps, NumOps};

/// A macro for quickly creating a new Vector2 with `x` and `y` components.
/// If a `type` is given, it will cast the components to that type.
macro_rules! v2 {
    ($x:expr, $y:expr) => {
        Vector2::new($x, $y)
    };
    ($x:expr, $y:expr; $type:ty) => {
        Vector2::new($x as $type, $y as $type)
    };
}

pub(crate) use v2;

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    pub const fn new(x: T, y: T) -> Self {
        Vector2 { x, y }
    }

    pub fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> T
    where
        T: Float,
    {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self)
    where
        T: Float,
    {
        let len = self.length();
        self.x = self.x / len;
        self.y = self.y / len;
    }

    pub fn normalized(&self) -> Self
    where
        T: Float,
    {
        let mut norm = *self;
        norm.normalize();
        norm
    }

    pub fn is_nan(&self) -> bool
    where
        T: Float,
    {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn flip_x(&mut self)
    where
        T: Neg<Output = T>,
    {
        self.x = -self.x;
    }

    pub fn flip_y(&mut self)
    where
        T: Neg<Output = T>,
    {
        self.y = -self.y;
    }

    pub fn dot(&self, other: Vector2<T>) -> T {
        self.x * other.x + self.y * other.y
    }

    /// Calculates the reflection (bounce) of this vector with respect to the `surface_normal`.
    pub fn reflect(&self, surface_normal: Vector2<T>) -> Vector2<T>
    where
        T: Float,
    {
        let lhs = surface_normal * self.dot(surface_normal);
        // Hacky way to do: `lhs * 2`, but stay generic
        *self - (lhs + lhs)
    }

    /// Returns a vector that is perpendicular to this one. Not to be mistaken with normalize!
    pub fn normal(&self) -> Vector2<T>
    where
        T: Neg<Output = T>,
    {
        Vector2::new(-self.y, self.x)
    }

    /// Absolute value of this vector. Makes both components positive (both components now have
    /// their absolute value).
    pub fn abs(&self) -> Vector2<T>
    where
        T: Float,
    {
        Vector2::new(self.x.abs(), self.y.abs())
    }

    /// Creates a random unit length vector
    pub fn random_unit() -> Vector2<f32> {
        let x = fastrand::f32();
        let y = fastrand::f32();
        Vector2::new(x, y).normalized()
    }

    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }

    pub fn cross(&self, other: Vector2<T>) -> T {
        self.x * other.y - self.y * other.x
    }
}

impl<T> Add for Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign for Vector2<T>
where
    T: Num + NumOps + NumAssignOps + Copy + Default,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T> Sub for Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> SubAssign for Vector2<T>
where
    T: Num + NumOps + NumAssignOps + Copy + Default,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Vector2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector2<T>
where
    T: Num + NumOps + NumAssignOps + Copy + Default,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T> Div<T> for Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Vector2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector2<T>
where
    T: Num + NumOps + NumAssignOps + Copy + Default,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T> Sum for Vector2<T>
where
    T: Num + NumOps + Copy + Default,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vector2::zero(), |acc, x| acc + x)
    }
}

#[cfg(test)]
mod tests {
    use super::Vector2;

    #[test]
    fn reflection_same_side() {
        let vector = v2!(3, -3; f32);
        let surface_normal = v2!(0, 1; f32);

        let reflected = vector.reflect(surface_normal);

        assert_eq!(reflected, v2!(3, 3; f32))
    }

    #[test]
    fn reflection_opposite_side() {
        let vector = v2!(3, -3; f32);
        let surface_normal = v2!(0, -1; f32);

        let reflected = vector.reflect(surface_normal);

        assert_eq!(reflected, v2!(3, 3; f32))
    }
}
