use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    Matrix, Result, SolverError,
};

use super::supply_token::SupplyToken;

#[derive(Debug, Clone)]
pub(crate) struct NetworkState {
    pub(crate) scenario_id: usize,

    pub(crate) fixed_arcs: Vec<(usize, usize)>,
    pub(crate) relative_draws: HashMap<(usize, usize), i64>,

    pub(crate) capacities: Matrix<usize>,
    pub(crate) costs: Arc<Matrix<usize>>,

    pub(crate) arc_loads: Matrix<usize>,
}

impl NetworkState {
    fn refresh(&mut self, token: &SupplyToken) -> Result<(Matrix<usize>, Matrix<usize>)> {
        log::debug!(
            "({}): Performing scheduled refresh for token {}.",
            self.scenario_id,
            token,
        );
        let (distance_map, predecessor_map) = floyd_warshall(
            &self.capacities.apply_mask(&token.intermediate_arc_set, 0),
            &self.costs,
        );
        let successor_map = invert_predecessors(&predecessor_map)?;

        Ok((distance_map, successor_map))
    }

    pub(crate) fn use_arc(&mut self, token: &mut SupplyToken, next_vertex: usize) {
        token.intermediate_arc_set.set(token.s, next_vertex, false);
        let _ = self.arc_loads.increment(token.s, next_vertex);
        let remaining_capacity = self.capacities.decrement(token.s, next_vertex);
        if remaining_capacity == 0 {
            log::info!(
                "({}): Arc ({}->{}) has reached its capacity. Tokens will be refreshed lazily.",
                self.scenario_id,
                token.s,
                next_vertex,
            );
        }
    }

    fn get_closest_fixed_arc(&self, token: &SupplyToken) -> Option<(usize, usize)> {
        self.fixed_arcs
            .iter()
            .min_by_key(|(a_0, a_1)| {
                let dist = (*token.distances.get(token.s, *a_0))
                    .saturating_add(*self.costs.get(*a_0, *a_1))
                    .saturating_add(*token.distances.get(*a_1, token.t));
                if dist == usize::MAX {
                    dist as i64
                } else {
                    (dist as i64)
                        .saturating_sub(*self.relative_draws.get(&(*a_0, *a_1)).unwrap_or(&0))
                }
            })
            .copied()
    }

    pub(crate) fn get_next_vertex(&mut self, token: &mut SupplyToken) -> Result<usize> {
        if token.needs_refresh(&self.capacities) {
            (token.distances, token.successors) = self.refresh(token)?;
        }

        let next_vertex_via_direct_path = *token.successors.get(token.s, token.t);
        if next_vertex_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(self.scenario_id));
        }

        let closest_fixed_arc = match self.get_closest_fixed_arc(token) {
            Some(fixed_arc) => fixed_arc,
            None => return Ok(next_vertex_via_direct_path),
        };

        // dist(v, v) is always 0 thanks to Floyd-Warshall!
        let next_vertex_via_fixed_arc = if token.s == closest_fixed_arc.0 {
            *token.successors.get(token.s, closest_fixed_arc.1)
        } else {
            *token.successors.get(token.s, closest_fixed_arc.0)
        };
        if next_vertex_via_fixed_arc == usize::MAX {
            return Ok(next_vertex_via_direct_path);
        }

        let cost_via_direct_path = *token.distances.get(token.s, token.t);
        if cost_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(self.scenario_id));
        }

        let cost_via_fixed_arc = token
            .distances
            .get(token.s, closest_fixed_arc.0)
            .saturating_add(*self.costs.get(closest_fixed_arc.0, closest_fixed_arc.1))
            .saturating_add(*token.distances.get(closest_fixed_arc.1, token.t));
        if cost_via_fixed_arc == usize::MAX {
            return Ok(next_vertex_via_direct_path);
        }

        if (cost_via_direct_path as i64)
            < (cost_via_fixed_arc as i64)
                - *self.relative_draws.get(&closest_fixed_arc).unwrap_or(&0)
        {
            Ok(next_vertex_via_direct_path)
        } else {
            Ok(next_vertex_via_fixed_arc)
        }
    }
}
