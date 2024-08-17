use crate::{matrix::Matrix, shortest_path};
use serde::Deserialize;
use serde_json;
use std::{fmt::Display, fs::File, io::BufReader, sync::Arc};

#[derive(Deserialize, Debug)]
pub struct Network {
    pub vertices: Vec<String>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,
    #[serde(skip)]
    pub arc_loads: Option<Vec<Matrix<usize>>>,
    #[serde(skip)]
    auxiliary_network: Option<Box<AuxiliaryNetwork>>,
}

#[derive(Debug)]
struct AuxiliaryNetwork {
    num_vertices: usize,
    costs: Matrix<usize>,
    scenarios: Vec<Box<Scenario>>,
    fixed_arcs: Vec<(usize, usize)>,
    fixed_arcs_memory: Vec<(usize, usize)>,
    intermediate_arc_sets: Matrix<Arc<Matrix<bool>>>,
}

#[derive(Debug)]
struct Scenario {
    capacities: Matrix<usize>,
    b_tuples_free: Vec<Box<BTuple>>,
    b_tuples_fixed: Vec<Vec<Box<BTuple>>>,
    successor_map: Matrix<usize>,
    distance_map: Matrix<usize>,
}

#[derive(Debug, Clone)]
struct BTuple {
    s: usize,
    t: usize,
    intermediate_arc_set: Arc<Matrix<bool>>,
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
        let len = self.vertices.len();

        let matrices = [&self.capacities, &self.costs];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices u, c have differing dimensions or are not quadratic");
            }
        }

        for matrix in &self.balances {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices in b have differing dimensions or are not quadratic");
            }
        }
    }

    fn preprocess(mut self) {
        match self.auxiliary_network {
            Some(_) => {}
            None => self.auxiliary_network = Some(Box::new(AuxiliaryNetwork::from(&self))),
        }
    }
}

pub fn floyd_warshall(
    capacities: &Matrix<usize>,
    costs: &Matrix<usize>,
) -> (Matrix<usize>, Matrix<Option<usize>>) {
    let mut dist: Matrix<usize> =
        Matrix::filled_with(usize::MAX, costs.num_rows(), costs.num_columns());
    let mut prev: Matrix<Option<usize>> =
        Matrix::filled_with(None, costs.num_rows(), costs.num_columns());

    for (x, y) in capacities
        .indices()
        .filter(|(x, y)| *capacities.get(*x, *y) > 0)
    {
        log::debug!("dist {} -> {} is {}", x, y, costs.get(x, y));
        let _ = dist.set(x, y, *costs.get(x, y));
        let _ = prev.set(x, y, Some(x));
    }
    for v in 0..capacities.num_rows() {
        log::debug!("pred. of {} is {} with distance 0", v, v);
        let _ = dist.set(v, v, 0);
        let _ = prev.set(v, v, Some(v));
    }
    for k in 0..capacities.num_rows() {
        for i in 0..capacities.num_rows() {
            for j in 0..capacities.num_rows() {
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

impl From<&Network> for AuxiliaryNetwork {
    fn from(network: &Network) -> Self {
        let mut num_vertices = network.vertices.len();
        let mut fixed_arcs: Vec<(usize, usize)> = vec![];
        let mut fixed_arcs_memory: Vec<(usize, usize)> = vec![];
        let mut costs = network.costs.clone();
        let mut capacities = network.capacities.clone();
        let mut balances = network.balances.clone();
        let mut scenarios: Vec<Box<Scenario>> = vec![];

        for a in network.fixed_arcs.iter() {
            capacities = extend_matrix(&capacities, create_extension_vertex(&capacities, a.0, a.1));
            capacities.set(a.0, a.1, 0);

            costs = extend_matrix(&costs, create_extension_vertex(&costs, a.0, a.1));
            costs.set(a.0, a.1, 0);

            balances.iter_mut().for_each(|balance| {
                *balance = extend_matrix(&balance, create_extension_vertex(&balance, a.0, a.1));
                balance.set(a.0, a.1, 0);
            });

            num_vertices += 1;
            fixed_arcs.push((num_vertices, a.1));
            fixed_arcs_memory.push((num_vertices, a.0));
        }

        // while in later iterations, capacities can differ between (s, t) pairs in BTuples,
        // we can initially re-use distance and successor maps between all (s, t) pairs and
        // balances, since the arcs for the globally shortest path from s to t is guaranteed to
        // be included in in the intermediate arc set of (s, t).
        let (distance_map, predecessor_map) = floyd_warshall(&capacities, &costs);
        let successor_map = invert_predecessors(&predecessor_map);

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets = Matrix::from_elements(
            &intermediate_arc_sets(&distance_map, &capacities, |x| 2 * x)
                .elements()
                .map(|x| Arc::new(x.clone()))
                .collect(),
            num_vertices,
            num_vertices,
        ); // TODO: get d
           // from
           // somewehere...

        balances.iter().for_each(|balance| {
            let (b_tuples_free, b_tuples_fixed) =
                generate_b_tuples(&balance, &arc_sets, &fixed_arcs);
            scenarios.push(Box::new(Scenario {
                capacities: capacities.clone(),
                distance_map: distance_map.clone(),
                successor_map: successor_map.clone(),
                b_tuples_free,
                b_tuples_fixed,
            }));
        });

        AuxiliaryNetwork {
            num_vertices,
            costs,
            scenarios,
            fixed_arcs,
            fixed_arcs_memory,
            intermediate_arc_sets: arc_sets,
        }
    }
}

fn generate_b_tuples(
    balances: &Matrix<usize>,
    intermediate_arc_sets: &Matrix<Arc<Matrix<bool>>>,
    fixed_arcs: &Vec<(usize, usize)>,
) -> (Vec<Box<BTuple>>, Vec<Vec<Box<BTuple>>>) {
    let mut free: Vec<Box<BTuple>> = vec![];
    let mut fixed: Vec<Vec<Box<BTuple>>> = vec![vec![]; fixed_arcs.len()];
    balances
        .indices()
        .filter(|(s, t)| s != t && *balances.get(*s, *t) > 0)
        .for_each(|(s, t)| {
            let target = match fixed_arcs.iter().position(|&(a, _)| a == s) {
                Some(i) => &mut fixed[i],
                None => &mut free,
            };
            target.push(Box::new(BTuple {
                s,
                t,
                intermediate_arc_set: Arc::clone(intermediate_arc_sets.get(s, t)),
            }));
        });

    (free, fixed)
}

fn intermediate_arc_sets(
    dist: &Matrix<usize>,
    capacities: &Matrix<usize>,
    delta_fn: fn(usize) -> usize,
) -> Matrix<Matrix<bool>> {
    let m = dist.num_rows();
    let mut intermediate_arc_sets = Matrix::filled_with(Matrix::filled_with(false, m, m), m, m);

    for (s, t) in dist.indices() {
        if s == t {
            continue;
        }
        let max_path_length = delta(delta_fn, &dist, s, t);
        let arc_set = intermediate_arc_sets.get_mut(s, t);

        for (x, y) in dist.indices() {
            // ignores arcs that lead to s or away from t, as well as arcs with no capacity (i.e.
            // non-existent arcs)
            if x == y || y == s || x == t || *capacities.get(x, y) == 0 {
                continue;
            }
            let detour_length = dist.get(s, x) + dist.get(x, y) + dist.get(y, t);
            if detour_length <= max_path_length {
                arc_set.set(x, y, true);
            }
        }
    }
    intermediate_arc_sets
}

fn delta(delta_fn: fn(usize) -> usize, dist: &Matrix<usize>, s: usize, t: usize) -> usize {
    delta_fn(*dist.get(s, t))
}

pub fn invert_predecessors(prev: &Matrix<Option<usize>>) -> Matrix<usize> {
    let mut succ: Matrix<usize> =
        Matrix::filled_with(usize::MAX, prev.num_rows(), prev.num_columns());

    prev.indices().for_each(|(s, t)| {
        if *succ.get(s, t) == usize::MAX {
            let path = shortest_path(&prev, s, t);
            for i in 0..path.len() {
                succ.set(path[i], t, if i + 1 < path.len() { path[i + 1] } else { t });
                if *succ.get(path[i], t) != usize::MAX {
                    break;
                }
            }
        }
    });

    succ
}

fn create_extension_vertex(matrix: &Matrix<usize>, s: usize, t: usize) -> (Vec<usize>, Vec<usize>) {
    let mut new_row: Vec<usize> = vec![0; matrix.row_len()];
    new_row[t] = *matrix.get(s, t);

    let mut new_column = matrix.as_columns()[s].clone();
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
