use std::collections::HashMap;

use crate::{
    options::{DeltaFunction, RemainderSolveMethod},
    Matrix,
};

use super::b_tuple::BTuple;

pub(crate) fn generate_b_tuples(
    supply: &Matrix<usize>,
    remainder_method: RemainderSolveMethod,
    fixed_arc_count: usize,
    arc_sets: &Matrix<Matrix<bool>>,
) -> (Vec<BTuple>, HashMap<usize, Vec<BTuple>>) {
    let mut free: Vec<BTuple> = vec![];
    supply
        .indices()
        .filter(|(s, t)| s != t && *supply.get(*s, *t) > 0)
        .for_each(|(s, t)| {
            // Unless the remainder method is greedy, skip (s, t) pairs which cannot be routed via
            // at least one fixed arc
            match remainder_method {
                RemainderSolveMethod::Greedy => {}
                _ => {
                    let arc_set = arc_sets.get(s, t);
                    if !arc_set.as_columns()[arc_set.num_columns() - fixed_arc_count..]
                        .iter()
                        .any(|c| c.iter().any(|&e| e))
                    {
                        log::debug!("Skipped BTuple for ({s}, {t}) because it cannot be routed via any fixed arc.");
                        return;
                    }
                }
            }

            let b_tuple = BTuple { origin: s, s, t };

            log::debug!(
                "Generated {} BTuples for ({s}, {t}):\n{b_tuple}",
                *supply.get(s, t),
            );

            // we are working with single units of supply in order to prevent dead ends,
            // and initially, all supply is free
            let supply_at_s_t = vec![b_tuple; *supply.get(s, t)];
            free.extend(supply_at_s_t);
        });

    (free, HashMap::new())
}

pub(crate) fn generate_intermediate_arc_sets(
    dist: &Matrix<usize>,
    costs: &Matrix<usize>,
    capacities: &Matrix<usize>,
    delta_fn: &DeltaFunction,
) -> Matrix<Matrix<bool>> {
    let m = dist.num_rows();
    let mut arc_sets = Matrix::filled_with(Matrix::filled_with(false, m, m), m, m);

    for (s, t) in dist.indices() {
        if s == t || *dist.get(s, t) == usize::MAX {
            continue;
        }
        let max_path_length = delta(delta_fn, dist, s, t);
        let arc_set = arc_sets.get_mut(s, t);

        for (x, y) in dist.indices() {
            // ignores arcs that lead to s or away from t, as well as arcs with no capacity (i.e.
            // non-existent arcs) and unreachable arcs
            if x == y || y == s || x == t || *capacities.get(x, y) == 0 {
                continue;
            }
            let detour_length = dist
                .get(s, x)
                .saturating_add(*costs.get(x, y))
                .saturating_add(*dist.get(y, t));
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
    Matrix::from_elements(
        arc_sets.elements().cloned().collect::<Vec<_>>().as_slice(),
        m,
        m,
    )
}

fn delta(delta_fn: &DeltaFunction, dist: &Matrix<usize>, s: usize, t: usize) -> usize {
    delta_fn.apply(*dist.get(s, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Matrix<usize>, Matrix<usize>, Matrix<usize>) {
        let distance_map: Matrix<usize> = Matrix::from_elements(&[0, 2, 1, 1, 0, 2, 2, 1, 0], 3, 3);
        let costs: Matrix<usize> = Matrix::from_elements(&[0, 5, 1, 1, 0, 0, 0, 1, 0], 3, 3);
        let capacities: Matrix<usize> = Matrix::from_elements(&[0, 1, 1, 1, 0, 0, 0, 1, 0], 3, 3);

        (distance_map, costs, capacities)
    }

    #[test]
    fn test_generate_intermediate_arc_sets() {
        let (distance_map, costs, capacities) = setup();
        let expected_result_0_1: Matrix<bool> = Matrix::from_elements(
            &[false, false, true, false, false, false, false, true, false],
            3,
            3,
        );
        let actual_result = generate_intermediate_arc_sets(
            &distance_map,
            &costs,
            &capacities,
            &DeltaFunction::LinearMedium,
        );
        let actual_result_0_1 = actual_result.get(0, 1).clone();

        assert_eq!(expected_result_0_1, actual_result_0_1);
    }

    #[test]
    fn test_generate_b_tuples() {
        let supply: Matrix<usize> = Matrix::from_elements(&vec![0, 2, 1, 1, 0, 1, 0, 6, 0], 3, 3);

        let actual_result =
            generate_b_tuples(&supply, RemainderSolveMethod::Greedy, 0, &Matrix::empty());

        assert_eq!(11, actual_result.0.len());
        assert!(actual_result.1.is_empty());
    }
}
