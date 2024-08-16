use std::{
    fmt::{self, Display},
    fs::File,
    io::BufReader,
};

use serde::{Deserialize, Serialize};

mod matrix;
use matrix::Matrix;

fn main() {
    let n = read_from_file("network.json").unwrap();
    println!("Network: {:?}", n);

    let (dist, prev) = floyd_warshall(&n.u, &n.c);
    println!("Distances:");
    println!("{}", &dist);
    println!("Predecessors:");
    println!("{}", &prev);

    println!("Shortest path v1 -> v2: {:?}", shortest_path(&prev, 0, 1));
    println!("Shortest path v1 -> v3: {:?}", shortest_path(&prev, 0, 2));
    println!("Shortest path v2 -> v3: {:?}", shortest_path(&prev, 1, 2));

    let arc_sets = intermediate_arc_sets(&dist, &n.u, |x| 2 * x);
    println!("Intermediate Arc Sets:");
    for (s, t) in arc_sets.indices() {
        println!("A({}, {}) = ", s + 1, t + 1);
        println!("{}", arc_sets.get(s, t));
        println!("");
    }

    let b_tuples = generate_b_tuples((0, 2), &n.b, &arc_sets);
    for b_t in b_tuples.0 {
        println!("{}", b_t);
    }

    let m: Matrix<usize> = Matrix::from_rows(&vec![vec![]]);
    println!("{}", m);
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkRaw {
    v: Vec<String>,
    u: Vec<Vec<usize>>,
    c: Vec<Vec<usize>>,
    b: Vec<Vec<Vec<usize>>>,
    a_fix: Vec<(usize, usize)>,
}

#[derive(Clone, Debug)]
struct Network {
    v: Vec<String>,
    u: Matrix<usize>,
    c: Matrix<usize>,
    b: Vec<Matrix<usize>>,
    a_fix: Vec<(usize, usize)>,
}

#[derive(Debug)]
struct ExtendedNetwork {
    v: Vec<String>,
    u: Matrix<usize>,
    c: Matrix<usize>,
    b: Vec<Matrix<usize>>,
    a_fix: Vec<(usize, usize)>,
    extension_mappings: Vec<(usize, usize)>,
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

fn extend_matrix<T>(matrix: &Matrix<T>, row_col: (Vec<T>, Vec<T>)) -> Matrix<T>
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

#[derive(Clone)]
struct BTuple<'a> {
    s: usize,
    t: usize,
    supply: usize,
    lambda: usize,
    arc_set: &'a Matrix<bool>,
}

impl<'a> std::fmt::Display for BTuple<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(s={}, t={}, b(s,t)={}, λ={}, A(s,t)={:?})",
            self.s + 1,
            self.t + 1,
            self.supply,
            self.lambda,
            self.arc_set
                .indices()
                .filter(|(s, t)| *self.arc_set.get(*s, *t))
                .map(|(s, t)| (s + 1, t + 1))
                .collect::<Vec<(usize, usize)>>(),
        )
    }
}

fn greedy<'a>(
    mut b_tuples: Vec<BTuple<'a>>,
    mut waiting_at_a_fix: Vec<Vec<BTuple<'a>>>,
    n: &Network,
    dist: &Matrix<usize>,
    prev: &Matrix<Option<usize>>,
    a_fix: (usize, usize),
) -> Vec<Matrix<usize>> {
    let mut relative_attraction = vec![0; n.b.len()];
    let a_fix_cost = dist.get(a_fix.0, a_fix.1);
    while !b_tuples.is_empty() {
        let mut b_tuples_new: Vec<BTuple> = vec![];
        b_tuples.extend(get_consistent_flow_tuples(&mut waiting_at_a_fix));
        b_tuples.iter_mut().for_each(|b_t| {
            let path_cost_direct = *dist.get(b_t.s, b_t.t);
            let path_cost_via_a_fix =
                dist.get(b_t.s, a_fix.0) + a_fix_cost + dist.get(a_fix.1, b_t.t);
            let next_vertex =
                if path_cost_direct < path_cost_via_a_fix - relative_attraction[b_t.lambda] {
                    shortest_path(prev, b_t.s, b_t.t)[1]
                } else {
                    shortest_path(prev, b_t.s, a_fix.1)[1]
                };

            if next_vertex == b_t.t {
                b_t.supply = 0;
            }

            let mut b_t_new = b_t.clone();
            b_t_new.s = next_vertex;

            if next_vertex == a_fix.0 {
                waiting_at_a_fix[b_t.lambda].push(b_t_new);
            } else {
                b_tuples_new.push(b_t_new);
            }
            b_t.supply = 0;
        });
        b_tuples.extend(b_tuples_new);
        waiting_at_a_fix.iter_mut().for_each(|a_fix_b_tuples| {
            a_fix_b_tuples.retain(|b_t| b_t.supply > 0);
        });
        let mut scenario_supplies: Vec<usize> = vec![0; n.b.len()];
        b_tuples = b_tuples
            .into_iter()
            .filter(|b_t| b_t.supply > 0)
            .inspect(|b_t| {
                if b_t.s == a_fix.0 {
                    scenario_supplies[b_t.lambda] += b_t.supply
                }
            })
            .collect();

        let total_supply: usize = scenario_supplies.iter().sum();
        relative_attraction
            .iter_mut()
            .enumerate()
            .for_each(|(i, attr)| {
                *attr = total_supply - scenario_supplies[i];
            });
    }
    vec![]
}

fn get_consistent_flow_tuples<'a>(waiting_at_a_fix: &mut Vec<Vec<BTuple<'a>>>) -> Vec<BTuple<'a>> {
    let max_consistent_flow = waiting_at_a_fix
        .iter()
        .map(|b_tuples| b_tuples.iter().map(|b_t| b_t.supply).sum::<usize>())
        .min()
        .unwrap();
    let mut ready_to_send: Vec<BTuple> = vec![];
    waiting_at_a_fix.iter_mut().for_each(|b_tuples| {
        let mut remaining_flow = max_consistent_flow;
        b_tuples.iter_mut().for_each(|b_t| {
            let flow_to_send = if b_t.supply < remaining_flow {
                b_t.supply
            } else {
                remaining_flow
            };
            remaining_flow -= flow_to_send;
            let mut b_t_new = b_t.clone();
            b_t_new.supply = flow_to_send;
            b_t.supply -= flow_to_send;
            ready_to_send.push(b_t_new);
        });
    });
    ready_to_send
}

fn generate_b_tuples<'a>(
    a_fix: (usize, usize),
    balances: &Vec<Matrix<usize>>,
    arc_sets: &'a Matrix<Matrix<bool>>,
) -> (Vec<BTuple<'a>>, Vec<Vec<BTuple<'a>>>) {
    let mut b_tuples: Vec<BTuple<'a>> = vec![];
    let mut b_tuples_at_a_fix: Vec<Vec<BTuple<'a>>> = vec![vec![]; balances.len()];
    for (s, t) in arc_sets.indices().filter(|(s, t)| s != t) {
        let arc_set = arc_sets.get(s, t);
        if !arc_set.get(a_fix.0, a_fix.1) {
            // println!(
            //     "arc set for ({}, {}) does not contain ({}, {})",
            //     s + 1,
            //     t + 1,
            //     a_fix.0 + 1,
            //     a_fix.1 + 1
            // );
            continue;
        }
        for lambda in 0..balances.len() {
            let supply = *balances[lambda].get(s, t);
            // println!(
            //     "{} supplies {} with a supply of {} in scenario {}",
            //     s + 1,
            //     t + 1,
            //     supply,
            //     lambda
            // );
            if supply > 0 {
                let b_t = BTuple {
                    s,
                    t,
                    supply,
                    lambda,
                    arc_set,
                };
                if s == a_fix.0 {
                    b_tuples_at_a_fix[lambda].push(b_t);
                } else {
                    b_tuples.push(b_t);
                }
            }
        }
    }
    (b_tuples, b_tuples_at_a_fix)
}

fn delta(d: fn(usize) -> usize, dist: &Matrix<usize>, s: usize, t: usize) -> usize {
    d(*dist.get(s, t))
}

fn intermediate_arc_sets(
    dist: &Matrix<usize>,
    u: &Matrix<usize>,
    d: fn(usize) -> usize,
) -> Matrix<Matrix<bool>> {
    let n = dist.num_rows();
    let mut arc_sets = Matrix::filled_with(Matrix::filled_with(false, n, n), n, n);

    for (s, t) in dist.indices() {
        if s == t {
            continue;
        }
        let max_path_length = delta(d, &dist, s, t);
        let arc_set = arc_sets.get_mut(s, t);

        for (x, y) in dist.indices() {
            if x == y || y == s || x == t || *u.get(x, y) == 0 {
                continue;
            }
            let detour_length = dist.get(s, x) + dist.get(x, y) + dist.get(y, t);
            if detour_length <= max_path_length {
                let _ = arc_set.set(x, y, true);
            }
        }
    }
    arc_sets
}

fn floyd_warshall(u: &Matrix<usize>, c: &Matrix<usize>) -> (Matrix<usize>, Matrix<Option<usize>>) {
    let mut dist: Matrix<usize> = Matrix::filled_with(usize::MAX, c.num_rows(), c.num_columns());
    let mut prev: Matrix<Option<usize>> = Matrix::filled_with(None, c.num_rows(), c.num_columns());

    for (x, y) in u.indices().filter(|(x, y)| *u.get(*x, *y) > 0) {
        // println!("dist {} -> {} is {}", x+1, y+1, c.get(x, y));
        let _ = dist.set(x, y, *c.get(x, y));
        let _ = prev.set(x, y, Some(x));
    }
    for v in 0..u.num_rows() {
        // println!("pred. of {} is {} with distance 0", v+1, v+1);
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
                    // println!("new dist {} -> {} is {}", i+1, j+1, dist.get(i, k).unwrap() + dist.get(k, j).unwrap());
                    let _ = dist.set(i, j, dist.get(i, k) + dist.get(k, j));
                    let _ = prev.set(i, j, *prev.get(k, j));
                }
            }
        }
    }

    (dist, prev)
}

fn shortest_path(prev: &Matrix<Option<usize>>, s: usize, mut t: usize) -> Vec<usize> {
    let mut p = match prev.get(s, t) {
        Some(_) => vec![t],
        None => return vec![],
    };

    while s != t {
        t = prev.get(s, t).expect("");
        p.push(t);
    }

    p.reverse();
    p
}

fn read_from_file(filename: &str) -> Result<Network, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let network_raw: NetworkRaw = serde_json::from_reader(reader)?;

    let network = Network {
        v: network_raw.v,
        u: Matrix::from_rows(&network_raw.u),
        c: Matrix::from_rows(&network_raw.c),
        b: network_raw
            .b
            .into_iter()
            .map(|b| Matrix::from_rows(&b))
            .collect::<Vec<_>>(),
        a_fix: network_raw.a_fix,
    };

    validate_network(&network)?;

    Ok(network)
}

#[derive(Debug)]
struct NetworkShapeError {
    msg: String,
}
impl NetworkShapeError {
    pub fn new(msg: &str) -> NetworkShapeError {
        NetworkShapeError {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for NetworkShapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for NetworkShapeError {}

impl From<&str> for NetworkShapeError {
    fn from(msg: &str) -> Self {
        NetworkShapeError::new(msg)
    }
}

fn validate_network(n: &Network) -> Result<(), NetworkShapeError> {
    let len = n.v.len();

    let matrices = [&n.u, &n.c];
    for matrix in matrices {
        if matrix.num_rows() != len || matrix.num_columns() != len {
            return Err("Matrices u, c have differing dimensions or are not quadratic")?;
        }
    }

    for matrix in &n.b {
        if matrix.num_rows() != len || matrix.num_columns() != len {
            return Err("Matrices in b have differing dimensions or are not quadratic")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_matrix() {
        let initial = vec![vec![1, 2], vec![4, 5]];
        assert_eq!(
            extend_matrix(&Matrix::from_rows(&initial), (vec![7, 8], vec![3, 6, 9])).as_rows(),
            vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]
        );
    }
}
