use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use num_traits::{Float, Num, NumAssignOps, NumOps};

#[derive(Copy, Clone, Debug, Default)]
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
