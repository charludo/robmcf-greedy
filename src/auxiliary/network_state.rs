use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    Matrix, Result, SolverError,
};

#[derive(Debug, Clone)]
pub(crate) struct NetworkState {
    pub(crate) intermediate_arc_sets: Matrix<Matrix<bool>>,
    pub(crate) fixed_arcs: Vec<(usize, usize)>,

    pub(crate) distances: Matrix<Matrix<usize>>,
    pub(crate) successors: Matrix<Matrix<usize>>,

    pub(crate) capacities: Matrix<usize>,
    pub(crate) costs: Arc<Matrix<usize>>,

    pub(crate) arc_loads: Matrix<usize>,

    pub(crate) relative_draws: HashMap<(usize, usize), i64>,
    pub(crate) needs_refresh: Matrix<bool>,
}

impl NetworkState {
    fn refresh(&mut self, origin: usize, dest: usize) -> Result<()> {
        log::debug!("Performing scheduled refresh for (s, t) pair ({origin}, {dest}).");
        let (distance_map, predecessor_map) = floyd_warshall(
            &self
                .capacities
                .apply_mask(self.intermediate_arc_sets.get(origin, dest), 0),
            &self.costs,
        );
        let successor_map = invert_predecessors(&predecessor_map);

        self.distances.set(origin, dest, distance_map);
        self.successors.set(origin, dest, successor_map?);

        self.needs_refresh.set(origin, dest, false);
        Ok(())
    }

    fn schedule_refresh(&mut self, s: usize, t: usize) {
        log::info!("Arc ({s}->{t}) has reached its capacity. Scheduling refreshes for all affected (s, t) pairs.");
        let affected_indices = self
            .intermediate_arc_sets
            .indices()
            .filter(|(origin, dest)| *self.intermediate_arc_sets.get(*origin, *dest).get(s, t));
        affected_indices.for_each(|(origin, dest)| {
            log::debug!("-> ({origin}, {dest})");
            self.needs_refresh.set(origin, dest, true);
        });
    }

    pub(crate) fn use_arc(&mut self, s: usize, t: usize) {
        let _ = self.arc_loads.increment(s, t);
        let remaining_capacity = self.capacities.decrement(s, t);
        if remaining_capacity == 0 {
            self.schedule_refresh(s, t);
        }
    }

    fn get_closest_fixed_arc(
        &self,
        origin: usize,
        s: usize,
        dest: usize,
    ) -> Option<(usize, usize)> {
        let distances = self.distances.get(origin, dest);
        self.fixed_arcs
            .iter()
            .min_by_key(|(a_0, a_1)| {
                ((*distances.get(s, *a_0) as i64)
                    + (*self.costs.get(*a_0, *a_1) as i64)
                    + (*distances.get(*a_1, dest) as i64))
                    .saturating_sub(*self.relative_draws.get(&(*a_0, *a_1)).unwrap_or(&0))
            })
            .copied()
    }

    pub(crate) fn get_next_vertex(
        &mut self,
        scenario_id: usize,
        origin: usize,
        s: usize,
        dest: usize,
    ) -> Result<usize> {
        if *self.needs_refresh.get(origin, dest) {
            self.refresh(origin, dest)?;
        }

        let successors = self.successors.get(origin, dest);
        let next_vertex_via_direct_path = *successors.get(s, dest);

        if next_vertex_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(scenario_id));
        }

        let closest_fixed_arc = match self.get_closest_fixed_arc(origin, s, dest) {
            Some(fixed_arc) => fixed_arc,
            None => return Ok(next_vertex_via_direct_path),
        };

        // dist(v, v) is always 0 thanks to Floyd-Warshall!
        let next_vertex_via_fixed_arc = if s == closest_fixed_arc.0 {
            *successors.get(s, closest_fixed_arc.1)
        } else {
            *successors.get(s, closest_fixed_arc.0)
        };
        if next_vertex_via_fixed_arc == usize::MAX {
            return Ok(next_vertex_via_direct_path);
        }

        let distances = self.distances.get(origin, dest);

        let cost_via_direct_path = *distances.get(s, dest);
        if cost_via_direct_path == usize::MAX {
            return Err(SolverError::NoFeasibleFlowError(scenario_id));
        }

        let cost_via_fixed_arc = distances
            .get(s, closest_fixed_arc.0)
            .saturating_add(*self.costs.get(closest_fixed_arc.0, closest_fixed_arc.1))
            .saturating_add(*distances.get(closest_fixed_arc.1, dest));
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
