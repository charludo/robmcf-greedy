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
    fn refresh_token(&mut self, token: &mut SupplyToken) -> Result<()> {
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

        token.distances = distance_map;
        token.successors = successor_map;
        Ok(())
    }

    pub(crate) fn use_arc(&mut self, token: &mut SupplyToken, next_vertex: usize) {
        token.intermediate_arc_set.set(token.s, next_vertex, false);

        let _ = self.arc_loads.increment(token.s, next_vertex);
        let remaining_capacity = self.capacities.decrement(token.s, next_vertex);
        if remaining_capacity == 0 {
            log::info!(
                "({}): Arc ({}->{}) has reached its capacity.",
                self.scenario_id,
                token.s,
                next_vertex,
            );
        }
    }

    fn get_closest_fixed_arc(&self, token: &SupplyToken) -> (usize, i64) {
        let mut vertex = usize::MAX;
        let mut cost = i64::MAX;

        for fixed_arc in &self.fixed_arcs {
            if !(*token.intermediate_arc_set.get(fixed_arc.0, fixed_arc.1)) {
                continue;
            }

            let cost_via_fixed_arc = token
                .distances
                .get(token.s, fixed_arc.0)
                .saturating_add(*self.costs.get(fixed_arc.0, fixed_arc.1))
                .saturating_add(*token.distances.get(fixed_arc.1, token.t));

            if cost_via_fixed_arc == usize::MAX {
                continue;
            }

            let mut cost_via_fixed_arc = cost_via_fixed_arc as i64;
            cost_via_fixed_arc = cost_via_fixed_arc
                .saturating_sub(*self.relative_draws.get(fixed_arc).unwrap_or(&0));

            if cost_via_fixed_arc >= cost {
                continue;
            }

            cost = cost_via_fixed_arc;
            vertex = if token.s == fixed_arc.0 {
                fixed_arc.1
            } else {
                *token.successors.get(token.s, fixed_arc.0)
            };
        }

        (vertex, cost)
    }

    pub(crate) fn get_next_vertex(&mut self, token: &mut SupplyToken) -> Result<usize> {
        self.refresh_token(token)?;

        let next_vertex_via_direct_path = *token.successors.get(token.s, token.t);
        if next_vertex_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(self.scenario_id));
        }

        let (next_vertex_via_fixed_arc, cost_via_fixed_arc) = self.get_closest_fixed_arc(token);
        if next_vertex_via_fixed_arc == usize::MAX || cost_via_fixed_arc == i64::MAX {
            return Ok(next_vertex_via_direct_path);
        }

        let cost_via_direct_path = *token.distances.get(token.s, token.t);
        if cost_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(self.scenario_id));
        }

        if (cost_via_direct_path as i64) < cost_via_fixed_arc {
            Ok(next_vertex_via_direct_path)
        } else {
            Ok(next_vertex_via_fixed_arc)
        }
    }
}
