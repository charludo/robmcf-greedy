use std::collections::HashMap;

use crate::matrix::Matrix;

use super::b_tuple::BTuple;

pub(super) fn generate_b_tuples(
    supply: &Matrix<usize>,
) -> (Vec<Box<BTuple>>, HashMap<usize, Vec<Box<BTuple>>>) {
    let mut free: Vec<Box<BTuple>> = vec![];
    supply
        .indices()
        .filter(|(s, t)| s != t && *supply.get(*s, *t) > 0)
        .for_each(|(s, t)| {
            let b_tuple = Box::new(BTuple { origin: s, s, t });

            log::debug!(
                "Generated {} BTuples for {} -> {}:\n{}",
                *supply.get(s, t),
                s,
                t,
                b_tuple,
            );

            // we are working with single units of supply in order to prevent dead ends,
            // and initially, all supply is free
            let supply_at_s_t = vec![b_tuple; *supply.get(s, t)];
            free.extend(supply_at_s_t);
        });

    (free, HashMap::new())
}

pub(super) fn generate_intermediate_arc_sets(
    dist: &Matrix<usize>,
    costs: &Matrix<usize>,
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
    Matrix::from_elements(&arc_sets.elements().map(|x| x.clone()).collect(), m, m)
}

fn delta(delta_fn: fn(usize) -> usize, dist: &Matrix<usize>, s: usize, t: usize) -> usize {
    delta_fn(*dist.get(s, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Matrix<usize>, Matrix<usize>, Matrix<usize>) {
        let distance_map: Matrix<usize> =
            Matrix::from_elements(&vec![0, 2, 1, 1, 0, 2, 2, 1, 0], 3, 3);
        let costs: Matrix<usize> = Matrix::from_elements(&vec![0, 5, 1, 1, 0, 0, 0, 1, 0], 3, 3);
        let capacities: Matrix<usize> =
            Matrix::from_elements(&vec![0, 1, 1, 1, 0, 0, 0, 1, 0], 3, 3);

        (distance_map, costs, capacities)
    }

    #[test]
    fn test_generate_intermediate_arc_sets() {
        let (distance_map, costs, capacities) = setup();
        let expected_result_0_1: Matrix<bool> = Matrix::from_elements(
            &vec![false, false, true, false, false, false, false, true, false],
            3,
            3,
        );
        let actual_result =
            generate_intermediate_arc_sets(&distance_map, &costs, &capacities, |x| 2 * x);
        let actual_result_0_1 = actual_result.get(0, 1).clone();

        assert_eq!(expected_result_0_1, actual_result_0_1);
    }

    #[test]
    fn test_generate_b_tuples() {
        let supply: Matrix<usize> = Matrix::from_elements(&vec![0, 2, 1, 1, 0, 1, 0, 6, 0], 3, 3);

        let actual_result = generate_b_tuples(&supply);

        assert_eq!(11, actual_result.0.len());
        assert!(actual_result.1.is_empty());
    }
}
