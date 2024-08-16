use core::fmt::Display;
use core::panic;

use array2d::Array2D;

pub struct Matrix<T>(Array2D<T>);

impl<T> Matrix<T>
where
    T: Copy + Display,
{
    fn get(&self, row: usize, column: usize) -> &T {
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

    fn get_mut(&mut self, row: usize, column: usize) -> &mut T {
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

    fn set(mut self, row: usize, column: usize, value: T) {
        match self.0.set(row, column, value) {
            Ok(_) => (),
            Err(msg) => panic!(
                "Attempted to set matrix element ({}, {}) to value {}, but encountered following error: {}",
                row,
                column,
                value,
                msg,
            ),
        }
    }

    fn from_rows(rows: &Vec<Vec<T>>) -> Self {
        match Array2D::from_rows(rows) {
            Ok(matrix) => Matrix(matrix),
            Err(msg) => panic!(
                "An error occurred while attempting to create a Matrix from rows: {}",
                msg
            ),
        }
    }

    fn as_rows(&self) -> Vec<Vec<T>> {
        self.0.as_rows()
    }

    fn as_columns(&self) -> Vec<Vec<T>> {
        self.0.as_columns()
    }

    fn indices(&self) -> impl DoubleEndedIterator + Iterator<Item = (usize, usize)> + Clone {
        self.0.indices_row_major()
    }
}
