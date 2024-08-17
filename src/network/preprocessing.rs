use std::{collections::HashMap, sync::Arc};

use crate::matrix::Matrix;

use super::b_tuple::BTuple;

pub(super) fn generate_b_tuples(
    balances: &Matrix<usize>,
    intermediate_arc_sets: &Matrix<Arc<Matrix<bool>>>,
    fixed_arcs: &Vec<(usize, usize)>,
) -> (Vec<Box<BTuple>>, HashMap<(usize, usize), Vec<Box<BTuple>>>) {
    let mut free: Vec<Box<BTuple>> = vec![];
    let mut fixed: HashMap<(usize, usize), Vec<Box<BTuple>>> = HashMap::new();
    balances
        .indices()
        .filter(|(s, t)| s != t && *balances.get(*s, *t) > 0)
        .for_each(|(s, t)| {
            let b_tuple = Box::new(BTuple {
                s,
                t,
                intermediate_arc_set: Arc::clone(intermediate_arc_sets.get(s, t)),
            });

            log::debug!(
                "Generated {} BTuples for {} -> {}:\n{}",
                *balances.get(s, t),
                s,
                t,
                b_tuple
            );

            // we are working with single units of supply i order to prevent dead ends
            let supply_at_s_t = vec![b_tuple; *balances.get(s, t)];
            match fixed_arcs.iter().position(|&(a, _)| a == s) {
                Some(_) => {
                    let _ = fixed.insert((s, t), supply_at_s_t);
                }
                None => free.extend(supply_at_s_t),
            };
        });

    (free, fixed)
}

pub(super) fn intermediate_arc_sets(
    dist: &Matrix<usize>,
    capacities: &Matrix<usize>,
    delta_fn: fn(usize) -> usize,
) -> Matrix<Matrix<bool>> {
    let m = dist.num_rows();
    let mut arc_sets = Matrix::filled_with(Matrix::filled_with(false, m, m), m, m);

    for (s, t) in dist.indices() {
        if s == t || *dist.get(s, t) == usize::MAX {
            continue;
        }
        let max_path_length = delta(delta_fn, &dist, s, t);
        let arc_set = arc_sets.get_mut(s, t);

        for (x, y) in dist.indices() {
            // ignores arcs that lead to s or away from t, as well as arcs with no capacity (i.e.
            // non-existent arcs) and unreachable arcs
            if x == y
                || y == s
                || x == t
                || *capacities.get(x, y) == 0
                || *dist.get(s, x) == usize::MAX
                || *dist.get(x, y) == usize::MAX
                || *dist.get(y, t) == usize::MAX
            {
                continue;
            }
            let detour_length = dist.get(s, x) + dist.get(x, y) + dist.get(y, t);
            if detour_length <= max_path_length {
                arc_set.set(x, y, true);
            }
        }

        log::debug!(
            "Generated the following intermediate arc set for ({}, {}):\n{}",
            s,
            t,
            arc_set
        );
    }
    arc_sets
}

fn delta(delta_fn: fn(usize) -> usize, dist: &Matrix<usize>, s: usize, t: usize) -> usize {
    delta_fn(*dist.get(s, t))
}
