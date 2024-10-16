mod display;
mod eq;
mod serde;

use core::fmt;
use std::fmt::Debug;

use array2d::Array2D;
use colored::{Color, ColoredString, Colorize};

#[derive(Debug, Clone)]
pub struct Matrix<T>(pub(super) Array2D<T>);

impl<T> Default for Matrix<T>
where
    T: Clone,
{
    fn default() -> Self {
        Matrix::empty()
    }
}

impl<T> Matrix<T> {
    pub fn get(&self, row: usize, column: usize) -> &T {
        match self.0.get(row, column) {
            Some(element) => element,
            None => panic!(
                "Attempted to access matrix element ({}, {}), but matrix has shape ({}, {})",
                row,
                column,
                &self.0.num_rows(),
                &self.0.num_columns()
            ),
        }
    }

    pub fn get_mut(&mut self, row: usize, column: usize) -> &mut T {
        let num_rows = self.0.num_rows();
        let num_columns = self.0.num_columns();
        match self.0.get_mut(row, column) {
            Some(element) => element,
            None => panic!(
                "Attempted to access matrix element ({}, {}), but matrix has shape ({}, {})",
                row, column, num_rows, num_columns,
            ),
        }
    }

    pub fn set(&mut self, row: usize, column: usize, value: T) {
        match self.0.set(row, column, value) {
            Ok(_) => (),
            Err(msg) => panic!(
                "Attempted to set matrix element ({}, {}), but encountered following error: {}",
                row, column, msg,
            ),
        }
    }

    pub fn from_rows(rows: &[Vec<T>]) -> Self
    where
        T: Clone,
    {
        match Array2D::from_rows(rows) {
            Ok(matrix) => Matrix(matrix),
            Err(msg) => panic!(
                "An error occurred while attempting to create a Matrix from rows: {}",
                msg
            ),
        }
    }

    pub fn filled_with(value: T, rows: usize, columns: usize) -> Self
    where
        T: Clone,
    {
        Matrix(Array2D::filled_with(value, rows, columns))
    }

    pub fn from_elements(elements: &[T], rows: usize, columns: usize) -> Self
    where
        T: Clone,
    {
        match Array2D::from_row_major(elements, rows, columns) {
            Ok(matrix) => Matrix(matrix),
            Err(msg) => panic!(
                "An error occurred while attempting to create a ({}, {}) matrix from a row: {}",
                rows, columns, msg
            ),
        }
    }

    pub fn empty() -> Self
    where
        T: Clone,
    {
        Matrix(Array2D::from_rows(&Vec::new()).unwrap())
    }

    pub fn apply_mask(&self, mask: &Matrix<bool>, bottom: T) -> Self
    where
        T: Clone,
    {
        assert!(self.num_rows() == mask.num_rows());
        assert!(self.num_columns() == mask.num_columns());

        Matrix::from_elements(
            self.elements()
                .zip(mask.elements())
                .map(|(x, m)| if *m { x.clone() } else { bottom.clone() })
                .collect::<Vec<T>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        )
    }

    pub fn extend(&mut self, row: &[T], column: &[T])
    where
        T: std::clone::Clone + Copy,
    {
        assert!(self.row_len() == row.len());
        assert!(self.column_len() == column.len() - 1);

        let mut matrix_unwrapped = self.as_rows();
        matrix_unwrapped.push(row.to_vec());
        for i in 0..column.len() {
            matrix_unwrapped[i].push(column[i]);
        }
        let _ = std::mem::replace(self, Matrix::<T>::from_rows(&matrix_unwrapped));
    }

    pub fn shrink(&mut self, amount: usize)
    where
        T: Clone,
    {
        assert!(self.row_len() > amount);
        assert!(self.column_len() > amount);

        let mut matrix_unwrapped = self.as_rows();
        matrix_unwrapped.truncate(self.row_len() - amount);
        for row in &mut matrix_unwrapped {
            row.truncate(self.column_len() - amount);
        }
        let _ = std::mem::replace(self, Matrix::<T>::from_rows(&matrix_unwrapped));
    }

    pub fn as_rows(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.0.as_rows()
    }

    pub fn as_columns(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.0.as_columns()
    }

    pub fn indices(&self) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone {
        self.0.indices_row_major()
    }

    pub fn elements(&self) -> impl DoubleEndedIterator<Item = &T> + Clone {
        self.0.elements_row_major_iter()
    }

    pub fn rows_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = impl DoubleEndedIterator<Item = &T> + Clone> + Clone {
        self.0.rows_iter()
    }

    pub fn row_len(&self) -> usize {
        self.0.row_len()
    }

    pub fn column_len(&self) -> usize {
        self.0.row_len()
    }

    pub fn num_rows(&self) -> usize {
        self.0.num_rows()
    }

    pub fn num_columns(&self) -> usize {
        self.0.num_columns()
    }
}

impl<T> Matrix<T>
where
    T: fmt::Display,
{
    pub(crate) fn highlight(
        &self,
        positions_to_highlight: &[(usize, usize)],
        color: Color,
    ) -> Matrix<ColoredString> {
        let mut highlighted: Matrix<ColoredString> = Matrix::from_elements(
            self.elements()
                .map(|x| x.to_string().white())
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        );
        positions_to_highlight.iter().for_each(|(s, t)| {
            highlighted.set(*s, *t, highlighted.get(*s, *t).clone().color(color));
        });
        highlighted
    }
}

impl Matrix<usize> {
    pub fn increment(&mut self, row: usize, column: usize) -> usize {
        let old = *self.get(row, column);
        if old < usize::MAX {
            self.set(row, column, old + 1);
        } else {
            log::error!("Attempted to increment with overflow. Aborted.");
            return old;
        }
        old + 1
    }
    pub fn decrement(&mut self, row: usize, column: usize) -> usize {
        let old = *self.get(row, column);
        if old > 0 {
            self.set(row, column, old - 1);
        } else {
            log::error!("Attempted to decrement with underflow. Aborted.");
            return old;
        }
        old - 1
    }

    pub fn sum(&self) -> usize {
        self.elements().sum()
    }

    pub fn add(&self, other: &Matrix<usize>) -> Self {
        assert!(self.num_rows() == other.num_rows());
        assert!(self.num_columns() == other.num_columns());

        Matrix::from_elements(
            self.elements()
                .zip(other.elements())
                .map(|(a, b)| a.saturating_add(*b))
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        )
    }

    pub fn subtract(&self, other: &Matrix<usize>) -> Self {
        assert!(self.num_rows() == other.num_rows());
        assert!(self.num_columns() == other.num_columns());

        Matrix::from_elements(
            self.elements()
                .zip(other.elements())
                .map(|(a, b)| a.saturating_sub(*b))
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        )
    }

    pub fn hadamard_product(&self, other: &Matrix<usize>) -> Self {
        assert!(self.num_rows() == other.num_rows());
        assert!(self.num_columns() == other.num_columns());

        let mut result_vec: Vec<usize> = vec![];
        self.indices().for_each(|(s, t)| {
            result_vec.push(self.get(s, t) * other.get(s, t));
        });
        Matrix::from_elements(&result_vec, self.num_rows(), self.num_columns())
    }

    pub fn min(&self) -> usize {
        *self.elements().min().unwrap()
    }

    pub fn max(&self) -> usize {
        *self.elements().max().unwrap()
    }
}

impl Matrix<i64> {
    pub fn subtract(&self, other: &Matrix<i64>) -> Self {
        assert!(self.num_rows() == other.num_rows());
        assert!(self.num_columns() == other.num_columns());

        Matrix::from_elements(
            self.elements()
                .zip(other.elements())
                .map(|(a, b)| a.saturating_sub(*b))
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        )
    }

    pub fn sum(&self) -> i64 {
        self.elements().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_mask() {
        let original: Matrix<usize> = Matrix::from_elements(&[1, 2, 3, 4], 2, 2);
        let mask: Matrix<bool> = Matrix::from_elements(&[true, false, false, true], 2, 2);
        let expected_result: Matrix<usize> =
            Matrix::from_elements(&[1, usize::MAX, usize::MAX, 4], 2, 2);
        assert_eq!(expected_result, original.apply_mask(&mask, usize::MAX));
    }

    #[test]
    fn test_extend() {
        let mut original: Matrix<usize> = Matrix::from_elements(&[1, 2, 4, 5], 2, 2);
        let row: Vec<usize> = vec![7, 8];
        let column: Vec<usize> = vec![3, 6, 9];
        let expected_result: Matrix<usize> =
            Matrix::from_elements(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3);

        original.extend(&row, &column);
        assert_eq!(expected_result, original);
    }

    #[test]
    fn test_shrink() {
        let mut original: Matrix<usize> = Matrix::from_elements(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3);
        let expected_result: Matrix<usize> = Matrix::from_elements(&[1, 2, 4, 5], 2, 2);

        original.shrink(1);
        assert_eq!(expected_result, original);
    }

    #[test]
    fn test_hadamard_product() {
        let original: Matrix<usize> = Matrix::from_elements(&[1, 2, 3, 4, 5, 6, 7, 8, 9], 3, 3);
        let expected_result: Matrix<usize> =
            Matrix::from_elements(&[1, 4, 9, 16, 25, 36, 49, 64, 81], 3, 3);

        assert_eq!(expected_result, original.hadamard_product(&original));
    }
}
