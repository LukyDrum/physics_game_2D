use std::ops::{Add, AddAssign, Mul};

use num_traits::Num;

use super::Vector2;

#[derive(Clone, PartialEq, Debug)]
pub struct Matrix<T, const R: usize, const C: usize>
where
    T: Copy + Clone,
{
    inner: [[T; C]; R],
    num_rows: usize,
    num_cols: usize,
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C>
where
    T: Copy + Clone,
{
    pub fn new(values: [[T; C]; R]) -> Self {
        Matrix {
            inner: values,
            num_rows: R,
            num_cols: C,
        }
    }

    pub fn get(&self, row: usize, col: usize) -> &T {
        &self.inner[row][col]
    }
}

impl Matrix<f32, 2, 2> {
    pub fn rotation_matrix(radians: f32) -> Self {
        let sin = radians.sin();
        let cos = radians.cos();

        Matrix::new([[cos, -sin], [sin, cos]])
    }
}

impl<T> From<Vector2<T>> for Matrix<T, 2, 1>
where
    T: Copy + Clone + Num + Default,
{
    fn from(vector: Vector2<T>) -> Self {
        Matrix::new([[vector.x], [vector.y]])
    }
}

/// Matrix addition
impl<T, const R: usize, const C: usize> Add<Matrix<T, R, C>> for Matrix<T, R, C>
where
    T: Copy + Clone + Add<Output = T> + Default,
{
    type Output = Matrix<T, R, C>;

    fn add(self, rhs: Matrix<T, R, C>) -> Self::Output {
        let mut new_inner = [[T::default(); C]; R];

        for (row_index, row) in new_inner.iter_mut().enumerate().take(R) {
            for (col_index, value) in row.iter_mut().enumerate().take(C) {
                *value = self.inner[row_index][col_index] + rhs.inner[row_index][col_index];
            }
        }

        Matrix::new(new_inner)
    }
}

/// Matrix multiplication
impl<T, const R: usize, const C: usize, const Q: usize> Mul<Matrix<T, C, Q>> for Matrix<T, R, C>
where
    T: Copy + Clone + Add + AddAssign + Mul<Output = T> + Default,
{
    type Output = Matrix<T, R, Q>;

    fn mul(self, rhs: Matrix<T, C, Q>) -> Self::Output {
        let mut new_inner = [[T::default(); Q]; R];

        for (row_index, row) in new_inner.iter_mut().enumerate().take(R) {
            for (col_index, value) in row.iter_mut().enumerate().take(Q) {
                let mut sum = T::default();
                for i in 0..C {
                    sum += self.inner[row_index][i] * rhs.inner[i][col_index];
                }
                *value = sum;
            }
        }

        Matrix::new(new_inner)
    }
}

/// Scalar multiplication
impl<T, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C>
where
    T: Copy + Clone + Mul<Output = T> + Default,
{
    type Output = Matrix<T, R, C>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut new_inner = [[T::default(); C]; R];

        for (row_index, row) in new_inner.iter_mut().enumerate().take(R) {
            for (col_index, value) in row.iter_mut().enumerate().take(C) {
                *value = self.inner[row_index][col_index] * rhs;
            }
        }

        Matrix::new(new_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn matrix_addition() {
        let mat_a = Matrix::new([[12, 15], [-5, 42]]);
        let mat_b = Matrix::new([[10, -5], [20, 8]]);

        let res = mat_a + mat_b;

        assert_eq!(res, Matrix::new([[22, 10], [15, 50],]))
    }

    #[test]
    fn scalar_multiplication() {
        let mat = Matrix::new([[12, 15], [-5, 42]]);

        let res = mat * 2;

        assert_eq!(res, Matrix::new([[24, 30], [-10, 84]]))
    }

    #[test]
    fn matrix_multiplication_same_dimensions() {
        let mat_a = Matrix::new([[12, 42], [-5, 10]]);
        let mat_b = Matrix::new([[3, 6], [9, 12]]);

        let res = mat_a * mat_b;

        assert_eq!(res, Matrix::new([[414, 576], [75, 90],]))
    }

    #[test]
    fn matrix_multiplication_different_dimensions() {
        let mat_a = Matrix::new([[12], [42]]);
        let mat_b = Matrix::new([[3, 6]]);

        let res = mat_a * mat_b;

        assert_eq!(res, Matrix::new([[36, 72], [126, 252],]))
    }
}
