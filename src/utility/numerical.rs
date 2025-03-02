use std::ops::{Add, Mul};

/// Should be much more accurate than explicit euler method.
///
/// Initial value problem: dy/dt = f(t, y); y(t_0) = y_0
///
/// Rate of change = f(t, y), eg: acceleration
pub fn runge_kutta<T>(current_value: T, step: f32, rate_of_change: T) -> T 
where T: Copy + Add<Output = T> + Mul<f32, Output = T> {
    let k1 = rate_of_change;
    let k2 = rate_of_change + (k1 * 0.5) * step;
    let k3 = rate_of_change + (k2 * 0.5) * step;
    let k4 = rate_of_change + k3 * step;

    current_value + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (step / 6.0) 
}
