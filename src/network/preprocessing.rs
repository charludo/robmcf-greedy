use std::{collections::HashMap, sync::Arc};

use crate::matrix::Matrix;

use super::b_tuple::BTuple;

pub(super) fn create_extension_vertex(
    matrix: &Matrix<usize>,
    s: usize,
    t: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut new_row: Vec<usize> = vec![0; matrix.row_len()];
    new_row[t] = *matrix.get(s, t);

    let mut new_column = matrix.as_columns()[s].clone();
    new_column.push(0);

    (new_row, new_column)
}

pub(super) fn generate_b_tuples(
    supply: &Matrix<usize>,
    intermediate_arc_sets: &Matrix<Arc<Matrix<bool>>>,
    fixed_arcs: &Vec<(usize, usize)>,
) -> (Vec<Box<BTuple>>, HashMap<(usize, usize), Vec<Box<BTuple>>>) {
    let mut free: Vec<Box<BTuple>> = vec![];
    let mut fixed: HashMap<(usize, usize), Vec<Box<BTuple>>> = HashMap::new();
    supply
        .indices()
        .filter(|(s, t)| s != t && *supply.get(*s, *t) > 0)
        .for_each(|(s, t)| {
            let b_tuple = Box::new(BTuple {
                s,
                t,
                intermediate_arc_set: Arc::clone(intermediate_arc_sets.get(s, t)),
            });

            log::debug!(
                "Generated {} BTuples for {} -> {}:\n{}",
                *supply.get(s, t),
                s,
                t,
                b_tuple
            );

            // we are working with single units of supply in order to prevent dead ends
            let supply_at_s_t = vec![b_tuple; *supply.get(s, t)];
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
    costs: &Matrix<usize>,
    capacities: &Matrix<usize>,
    delta_fn: fn(usize) -> usize,
) -> Matrix<Arc<Matrix<bool>>> {
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
            let detour_length = dist.get(s, x) + costs.get(x, y) + dist.get(y, t);
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
        &arc_sets.elements().map(|x| Arc::new(x.clone())).collect(),
        m,
        m,
    )
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
    fn test_create_extension_vertex() {
        let original: Matrix<usize> = Matrix::from_elements(&vec![1, 2, 3, 4], 2, 2);
        let fixed_arc: (usize, usize) = (1, 0);

        let expected_result = (vec![3, 0], vec![2, 4, 0]);
        let actual_result = create_extension_vertex(&original, fixed_arc.0, fixed_arc.1);
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn test_intermediate_arc_sets() {
        let (distance_map, costs, capacities) = setup();
        let expected_result_0_1: Matrix<bool> = Matrix::from_elements(
            &vec![false, false, true, false, false, false, false, true, false],
            3,
            3,
        );
        let actual_result = intermediate_arc_sets(&distance_map, &costs, &capacities, |x| 2 * x);
        let actual_result_0_1 = actual_result.get(0, 1).clone();

        assert_eq!(expected_result_0_1, *actual_result_0_1);
    }

    #[test]
    fn test_generate_b_tuples() {
        let (distance_map, costs, capacities) = setup();
        let arc_sets = intermediate_arc_sets(&distance_map, &costs, &capacities, |x| 2 * x); // yeah I'm not doing this by hand.
        let supply: Matrix<usize> = Matrix::from_elements(&vec![0, 2, 1, 1, 0, 1, 0, 6, 0], 3, 3);

        let actual_result = generate_b_tuples(&supply, &arc_sets, &vec![(2, 1)]);

        assert_eq!(5, actual_result.0.len());
        assert_eq!(6, actual_result.1.get(&(2, 1)).unwrap().len());
    }
}
