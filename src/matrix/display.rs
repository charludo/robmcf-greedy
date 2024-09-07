use core::fmt::Display;
use std::fmt::Result;

use colored::ColoredString;

use super::Matrix;

impl Display for Matrix<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let lpad = match self.elements().map(|x| x.len()).max() {
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

impl Display for Matrix<ColoredString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            Matrix::from_elements(
                &self.elements().map(|x| x.to_string()).collect::<Vec<_>>(),
                self.num_rows(),
                self.num_columns()
            )
        )
    }
}

impl Display for Matrix<usize> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let str_repr = Matrix::from_elements(
            self.elements()
                .map(|x| {
                    if *x == usize::MAX {
                        "?".to_string()
                    } else {
                        x.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        );
        write!(f, "{}", str_repr)
    }
}

impl Display for Matrix<bool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let str_repr = Matrix::from_elements(
            self.elements()
                .map(|x| (*x as usize).to_string())
                .collect::<Vec<_>>()
                .as_slice(),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let str_repr = Matrix::from_elements(
            self.elements()
                .map(|x| match x {
                    Some(e) => e.to_string(),
                    None => "?".to_string(),
                })
                .collect::<Vec<_>>()
                .as_slice(),
            self.num_rows(),
            self.num_columns(),
        );
        write!(f, "{}", str_repr)
    }
}
