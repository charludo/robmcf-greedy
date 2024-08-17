use std::fmt;

use log::{self, LevelFilter};
use std::io::Write;

mod matrix;
mod network;
use matrix::Matrix;
use network::*;

fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{style}{}{style:#} {}:{}] - {} ",
                record.level(),
                match record.file() {
                    Some(r) => r,
                    None => "",
                },
                match record.line() {
                    Some(r) => r.to_string(),
                    None => "".to_string(),
                },
                record.args()
            )
        })
        .init();
    let n = Network::from_file("network.json");
    println!("Network: {:?}", n);

    let (dist, prev) = floyd_warshall(&n.capacities, &n.costs);
    println!("Distances:");
    println!("{}", &dist);
    println!("Predecessors:");
    println!("{}", &prev);

    let succ = invert_predecessors(&prev);
    println!("Successors:");
    println!("{}", &succ);

    println!("Shortest path v1 -> v2: {:?}", shortest_path(&prev, 0, 1));
    println!("Shortest path v1 -> v3: {:?}", shortest_path(&prev, 0, 2));
    println!("Shortest path v2 -> v3: {:?}", shortest_path(&prev, 1, 2));

    let arc_sets = intermediate_arc_sets(&dist, &n.capacities, |x| 2 * x);
    println!("Intermediate Arc Sets:");
    for (s, t) in arc_sets.indices() {
        println!("A({}, {}) = ", s + 1, t + 1);
        println!("{}", arc_sets.get(s, t));
        println!("");
    }

    let b_tuples = generate_b_tuples((0, 2), &n.balances, &arc_sets);
    for b_t in b_tuples.0 {
        println!("{}", b_t);
    }

    let m: Matrix<usize> = Matrix::from_rows(&vec![vec![]]);
    println!("{}", m);
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
    let mut relative_attraction = vec![0; n.balances.len()];
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
        let mut scenario_supplies: Vec<usize> = vec![0; n.balances.len()];
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
            log::debug!(
                "arc set for ({}, {}) does not contain ({}, {})",
                s + 1,
                t + 1,
                a_fix.0 + 1,
                a_fix.1 + 1
            );
            continue;
        }
        for lambda in 0..balances.len() {
            let supply = *balances[lambda].get(s, t);
            log::debug!(
                "{} supplies {} with a supply of {} in scenario {}",
                s + 1,
                t + 1,
                supply,
                lambda
            );
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
