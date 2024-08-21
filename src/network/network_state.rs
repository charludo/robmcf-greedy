use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    matrix::Matrix,
};

pub(super) struct NetworkState {
    intermediate_arc_sets: Matrix<Matrix<bool>>,
    fixed_arcs: Matrix<Vec<usize>>,

    distances: Matrix<Matrix<usize>>,
    successors: Matrix<Matrix<usize>>,

    capacities: Matrix<usize>,
    costs: Arc<Matrix<usize>>,

    arc_loads: Matrix<usize>,
    slack: usize,
}

impl NetworkState {
    fn refresh(&mut self, s: usize, t: usize) {
        let affected_indices = self
            .intermediate_arc_sets
            .indices()
            .filter(|(origin, dest)| *self.intermediate_arc_sets.get(*origin, *dest).get(s, t));
        affected_indices.for_each(|(origin, dest)| {
            let (distance_map, predecessor_map) = floyd_warshall(
                &self
                    .capacities
                    .apply_mask(self.intermediate_arc_sets.get(origin, dest), usize::MAX),
                &self.costs,
            );
            let successor_map = invert_predecessors(&predecessor_map);

            self.distances.set(origin, dest, distance_map);
            self.successors.set(origin, dest, successor_map);
        });
    }

    fn use_arc(&mut self, origin: usize, dest: usize, s: usize, t: usize) {
        // Set the intermediate_arc_set to false for EVERY used arc.
        // This prevents loop.
        // We do NOT need to refresh any maps just because of this:
        // - other (s, t) pairs are not affected
        // - any shortest path is by definition acyclic
        // ...that being said, removing the arc from fixed_arcs makes to quicken future lookups
        self.intermediate_arc_sets
            .get_mut(origin, dest)
            .set(s, t, false);
        let _ = self.arc_loads.increment(s, t);
        let remaining_capacity = self.capacities.decrement(s, t);
        if remaining_capacity == 0 {
            self.refresh(s, t);
        }
    }

    fn use_fixed_arc(&mut self, origin: usize, dest: usize, s: usize, t: usize) {
        let fixed_arcs = self.fixed_arcs.get_mut(origin, dest);
        if let Some(index) = fixed_arcs.iter().find(|v| **v == s) {
            let _ = fixed_arcs.remove(*index);
        } else {
            log::error!("Attempted to use fixed arc ({s}->{t}), but it has already been used!");
        }
        self.use_arc(origin, dest, s, t);
    }

    fn get_closest_fixed_arc(
        &self,
        relative_draws: &HashMap<usize, usize>,
        origin: usize,
        s: usize,
        dest: usize,
    ) -> Option<usize> {
        // This assumes that the cost of the fixed arcs is non-zero, and the cost TO the fixed
        // vertex is zero instead.
        let distances = self.distances.get(origin, dest);
        self.fixed_arcs
            .get(origin, dest)
            .iter()
            .min_by_key(|fixed_arc| {
                *distances.get(s, **fixed_arc) + *distances.get(**fixed_arc, dest)
                    - *relative_draws.get(fixed_arc).unwrap_or(&0)
            })
            .copied()
    }

    fn get_next_vertex(
        &self,
        relative_draws: &HashMap<usize, usize>,
        origin: usize,
        s: usize,
        dest: usize,
    ) -> usize {
        let successors = self.successors.get(origin, dest);
        let next_vertex_via_direct_path = *successors.get(s, dest);
        let closest_fixed_arc = match self.get_closest_fixed_arc(relative_draws, origin, s, dest) {
            None => return next_vertex_via_direct_path,
            Some(fixed_arc) => fixed_arc,
        };
        let next_vertex_via_fixed_arc = *successors.get(s, closest_fixed_arc);

        let distances = self.distances.get(origin, dest);
        let cost_via_direct_path = *distances.get(s, dest);
        let cost_via_fixed_arc =
            *distances.get(s, closest_fixed_arc) + distances.get(closest_fixed_arc, dest);

        if cost_via_direct_path
            < cost_via_fixed_arc - *relative_draws.get(&closest_fixed_arc).unwrap_or(&0)
        {
            next_vertex_via_direct_path
        } else {
            next_vertex_via_fixed_arc
        }
    }
}
