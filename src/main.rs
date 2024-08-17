use std::fmt;

use log::{self, LevelFilter};
use std::io::Write;

mod matrix;
mod network;
use matrix::Matrix;
use network::*;

fn main() {
    env_logger::builder()
        .filter(None, LevelFilter::Debug)
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

    let m: Matrix<usize> = Matrix::from_rows(&vec![vec![]]);
    println!("{}", m);
}

// impl<'a> std::fmt::Display for BTuple<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(
//             f,
//             "(s={}, t={}, b(s,t)={}, λ={}, A(s,t)={:?})",
//             self.s + 1,
//             self.t + 1,
//             self.supply,
//             self.lambda,
//             self.arc_set
//                 .indices()
//                 .filter(|(s, t)| *self.arc_set.get(*s, *t))
//                 .map(|(s, t)| (s + 1, t + 1))
//                 .collect::<Vec<(usize, usize)>>(),
//         )
//     }
// }

// fn greedy<'a>(
//     mut b_tuples: Vec<BTuple<'a>>,
//     mut waiting_at_a_fix: Vec<Vec<BTuple<'a>>>,
//     n: &Network,
//     dist: &Matrix<usize>,
//     prev: &Matrix<Option<usize>>,
//     a_fix: (usize, usize),
// ) -> Vec<Matrix<usize>> {
//     let mut relative_attraction = vec![0; n.balances.len()];
//     let a_fix_cost = dist.get(a_fix.0, a_fix.1);
//     while !b_tuples.is_empty() {
//         let mut b_tuples_new: Vec<BTuple> = vec![];
//         b_tuples.extend(get_consistent_flow_tuples(&mut waiting_at_a_fix));
//         b_tuples.iter_mut().for_each(|b_t| {
//             let path_cost_direct = *dist.get(b_t.s, b_t.t);
//             let path_cost_via_a_fix =
//                 dist.get(b_t.s, a_fix.0) + a_fix_cost + dist.get(a_fix.1, b_t.t);
//             let next_vertex =
//                 if path_cost_direct < path_cost_via_a_fix - relative_attraction[b_t.lambda] {
//                     shortest_path(prev, b_t.s, b_t.t)[1]
//                 } else {
//                     shortest_path(prev, b_t.s, a_fix.1)[1]
//                 };
//
//             if next_vertex == b_t.t {
//                 b_t.supply = 0;
//             }
//
//             let mut b_t_new = b_t.clone();
//             b_t_new.s = next_vertex;
//
//             if next_vertex == a_fix.0 {
//                 waiting_at_a_fix[b_t.lambda].push(b_t_new);
//             } else {
//                 b_tuples_new.push(b_t_new);
//             }
//             b_t.supply = 0;
//         });
//         b_tuples.extend(b_tuples_new);
//         waiting_at_a_fix.iter_mut().for_each(|a_fix_b_tuples| {
//             a_fix_b_tuples.retain(|b_t| b_t.supply > 0);
//         });
//         let mut scenario_supplies: Vec<usize> = vec![0; n.balances.len()];
//         b_tuples = b_tuples
//             .into_iter()
//             .filter(|b_t| b_t.supply > 0)
//             .inspect(|b_t| {
//                 if b_t.s == a_fix.0 {
//                     scenario_supplies[b_t.lambda] += b_t.supply
//                 }
//             })
//             .collect();
//
//         let total_supply: usize = scenario_supplies.iter().sum();
//         relative_attraction
//             .iter_mut()
//             .enumerate()
//             .for_each(|(i, attr)| {
//                 *attr = total_supply - scenario_supplies[i];
//             });
//     }
//     vec![]
// }
