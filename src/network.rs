use crate::matrix::Matrix;
use serde::Deserialize;
use serde_json;
use std::{fmt::Display, fs::File, io::BufReader};

#[derive(Clone, Debug, Deserialize)]
pub struct Network {
    pub v: Vec<String>,
    pub u: Matrix<usize>,
    pub c: Matrix<usize>,
    pub b: Vec<Matrix<usize>>,
    pub a_fix: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct ExtendedNetwork {
    pub v: Vec<String>,
    pub u: Matrix<usize>,
    pub c: Matrix<usize>,
    pub b: Vec<Matrix<usize>>,
    pub a_fix: Vec<(usize, usize)>,
    pub extension_mappings: Vec<(usize, usize)>,
}

impl Network {
    pub fn from_file(filename: &str) -> Self {
        let file = match File::open(filename) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to open file \"{}\": {}", filename, msg),
        };
        let reader = BufReader::new(file);

        let network: Network = match serde_json::from_reader(reader) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to parse the network: {}", msg),
        };

        network.validate();

        network
    }

    fn validate(&self) {
        let len = self.v.len();

        let matrices = [&self.u, &self.c];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices u, c have differing dimensions or are not quadratic");
            }
        }

        for matrix in &self.b {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices in b have differing dimensions or are not quadratic");
            }
        }
    }
}

pub fn floyd_warshall(
    u: &Matrix<usize>,
    c: &Matrix<usize>,
) -> (Matrix<usize>, Matrix<Option<usize>>) {
    let mut dist: Matrix<usize> = Matrix::filled_with(usize::MAX, c.num_rows(), c.num_columns());
    let mut prev: Matrix<Option<usize>> = Matrix::filled_with(None, c.num_rows(), c.num_columns());

    for (x, y) in u.indices().filter(|(x, y)| *u.get(*x, *y) > 0) {
        log::debug!("dist {} -> {} is {}", x, y, c.get(x, y));
        let _ = dist.set(x, y, *c.get(x, y));
        let _ = prev.set(x, y, Some(x));
    }
    for v in 0..u.num_rows() {
        log::debug!("pred. of {} is {} with distance 0", v, v);
        let _ = dist.set(v, v, 0);
        let _ = prev.set(v, v, Some(v));
    }
    for k in 0..u.num_rows() {
        for i in 0..u.num_rows() {
            for j in 0..u.num_rows() {
                if *dist.get(i, k) < usize::MAX
                    && *dist.get(k, j) < usize::MAX
                    && *dist.get(i, j) > dist.get(i, k) + dist.get(k, j)
                {
                    log::debug!(
                        "new dist {} -> {} is {}",
                        i,
                        j,
                        dist.get(i, k) + dist.get(k, j)
                    );
                    let _ = dist.set(i, j, dist.get(i, k) + dist.get(k, j));
                    let _ = prev.set(i, j, *prev.get(k, j));
                }
            }
        }
    }

    (dist, prev)
}

impl From<Network> for ExtendedNetwork {
    fn from(n: Network) -> Self {
        let a_fix = n.a_fix;
        let mut n = ExtendedNetwork {
            v: n.v,
            u: n.u,
            c: n.c,
            b: n.b,
            a_fix: vec![],
            extension_mappings: vec![],
        };
        for a in a_fix.iter() {
            let k = n.v.len();
            n.v.push(format!("(v{}={}->{})", k + 1, n.v[a.0], n.v[a.1]));

            n.u = extend_matrix(&n.u, create_extension_vertex(&n.u, a.0, a.1));
            n.u.set(a.0, a.1, 0);

            n.c = extend_matrix(&n.c, create_extension_vertex(&n.c, a.0, a.1));
            n.c.set(a.0, a.1, 0);

            let mut new_b: Vec<Matrix<usize>> = vec![];
            for b in &n.b {
                new_b.push(extend_matrix(
                    &b,
                    (vec![0; b.row_len()], vec![0; b.column_len() + 1]),
                ));
            }

            n.a_fix.push((k + 1, a.1));
            n.extension_mappings.push((k + 1, a.0));
        }

        n
    }
}

fn create_extension_vertex(matrix: &Matrix<usize>, s: usize, t: usize) -> (Vec<usize>, Vec<usize>) {
    let mut new_row: Vec<usize> = vec![];
    for i in 0..matrix.row_len() {
        if i == t {
            new_row.push(*matrix.get(s, t));
            continue;
        }
        new_row.push(0);
    }

    let mut new_column = matrix.as_columns()[t].clone();
    new_column.push(0);

    (new_row, new_column)
}

pub fn extend_matrix<T>(matrix: &Matrix<T>, row_col: (Vec<T>, Vec<T>)) -> Matrix<T>
where
    T: std::clone::Clone + Display + Copy,
{
    assert!(matrix.row_len() == row_col.0.len());
    assert!(matrix.column_len() == row_col.1.len() - 1);

    let mut matrix_unwrapped = matrix.as_rows();
    matrix_unwrapped.push(row_col.0.clone());
    for i in 0..row_col.1.len() {
        matrix_unwrapped[i].push(row_col.1[i].clone());
    }
    Matrix::<T>::from_rows(&matrix_unwrapped)
}
