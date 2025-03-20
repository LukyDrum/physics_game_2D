use std::ops::{Add, Div, Mul};

use num_traits::Num;

/// Should be much more accurate than explicit euler method.
///
/// Initial value problem: dy/dt = f(t, y); y(t_0) = y_0
///
/// Rate of change = f(t, y), eg: acceleration
pub fn runge_kutta<T>(current_value: T, step: f32, rate_of_change: T) -> T
where
    T: Copy + Add<Output = T> + Mul<f32, Output = T>,
{
    let k1 = rate_of_change;
    let k2 = rate_of_change + (k1 * 0.5) * step;
    let k3 = rate_of_change + (k2 * 0.5) * step;
    let k4 = rate_of_change + k3 * step;

    current_value + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (step / 6.0)
}

/// Calculates the average of the slice only from values which are non-zero.
pub fn non_zero_average<T>(values: &[T], threshold: T) -> T
where
    T: Add + Div<Output = T> + Num + Copy + PartialOrd,
{
    let mut sum = T::zero();
    let mut count = T::zero();
    for x in values {
        if *x > threshold {
            count = count + T::one();
            sum = sum + *x;
        }
    }

    if count.is_zero() {
        return sum;
    }

    sum / count
}
