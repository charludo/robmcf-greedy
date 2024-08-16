use core::fmt::Display;
use core::panic;
use std::fmt::Debug;

use array2d::Array2D;

#[derive(Debug, Clone)]
pub struct Matrix<T>(Array2D<T>);

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

    pub fn from_rows(rows: &Vec<Vec<T>>) -> Self
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

    pub fn from_elements(elements: &Vec<T>, rows: usize, columns: usize) -> Self
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

    pub fn indices(&self) -> impl DoubleEndedIterator + Iterator<Item = (usize, usize)> + Clone {
        self.0.indices_row_major()
    }

    pub fn elements(&self) -> impl DoubleEndedIterator<Item = &T> + Clone {
        self.0.elements_row_major_iter()
    }

    pub fn rows_iter(
        &self,
    ) -> impl DoubleEndedIterator
           + Iterator<Item = impl DoubleEndedIterator + Iterator<Item = &T> + Clone>
           + Clone {
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

impl Display for Matrix<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lpad = match self.elements().map(|x| x.to_string().len()).max() {
            Some(element) => element,
            None => return write!(f, "[[]]"),
        };
        let num_rows = self.0.num_rows();
        let num_columns = self.0.num_columns();
        let mut string_repr: Vec<String> = vec![];
        self.rows_iter().enumerate().for_each(|(i, row)| {
            if i != 0 {
                string_repr.push(" ".to_string());
            } else {
                string_repr.push("[".to_string());
            }
            string_repr.push("[".to_string());
            row.enumerate().for_each(|(j, elem)| {
                string_repr.push(format!("{:>lpad$}", elem, lpad = lpad).to_string());
                if j != num_columns - 1 {
                    string_repr.push(", ".to_string());
                }
            });
            string_repr.push("]".to_string());
            if i == num_rows - 1 {
                string_repr.push("]".to_string());
            } else {
                string_repr.push("\n".to_string());
            }
        });
        write!(f, "{}", string_repr.join(""))
    }
}

impl Display for Matrix<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr = Matrix::from_elements(
            &self.elements().map(|x| x.to_string()).collect(),
            self.num_rows(),
            self.num_columns(),
        );
        write!(f, "{}", str_repr)
    }
}

impl Display for Matrix<bool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr = Matrix::from_elements(
            &self.elements().map(|x| (*x as usize).to_string()).collect(),
            self.num_rows(),
            self.num_columns(),
        );
        write!(f, "{}", str_repr)
    }
}

impl<T> Display for Matrix<Option<T>>
where
    T: ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr = Matrix::from_elements(
            &self
                .elements()
                .map(|x| match x {
                    Some(e) => e.to_string(),
                    None => "?".to_string(),
                })
                .collect(),
            self.num_rows(),
            self.num_columns(),
        );
        write!(f, "{}", str_repr)
    }
}
