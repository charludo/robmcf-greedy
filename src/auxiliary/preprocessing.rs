use crate::{
    options::{DeltaFunction, RemainderSolveMethod},
    Matrix,
};

use super::supply_token::SupplyToken;

pub(crate) fn generate_supply_tokens(
    supply: &Matrix<usize>,
    fixed_arcs: &[(usize, usize)],
    remainder_method: RemainderSolveMethod,
    arc_sets: &Matrix<Matrix<bool>>,
) -> Vec<SupplyToken> {
    let mut tokens: Vec<SupplyToken> = vec![];
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
                    if !fixed_arcs
                        .iter()
                        .any(|(a_0, a_1)| *arc_set.get(*a_0, *a_1))
                    {
                        log::debug!("Skipped SupplyToken for ({s}, {t}) because it cannot be routed via any fixed arc.");
                        return;
                    }
                }
            }

            let token = SupplyToken { origin: s, s, t };
            log::debug!("{}x {token}", *supply.get(s, t));

            // we are working with single units of supply in order to prevent dead ends
            let supply_at_s_t = vec![token; *supply.get(s, t)];
            tokens.extend(supply_at_s_t);
        });

    tokens
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

        log::trace!(
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
    fn test_generate_supply_tokens() {
        let supply: Matrix<usize> = Matrix::from_elements(&[0, 2, 1, 1, 0, 1, 0, 6, 0], 3, 3);

        let actual_result =
            generate_supply_tokens(&supply, &[], RemainderSolveMethod::Greedy, &Matrix::empty());

        assert_eq!(11, actual_result.len());
    }
}
