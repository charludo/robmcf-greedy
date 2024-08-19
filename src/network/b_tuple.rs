use std::{collections::HashMap, sync::Arc};

use crate::matrix::Matrix;

#[derive(Debug, Clone)]
pub(crate) struct BTuple {
    pub(crate) s: usize,
    pub(crate) t: usize,
    pub(crate) intermediate_arc_set: Arc<Matrix<bool>>,
    pub(crate) successor_map: Matrix<usize>,
    pub(crate) distance_map: Matrix<usize>,
    pub(super) fixed_arc_distances: HashMap<(usize, usize), usize>,
}

impl BTuple {
    pub(crate) fn generate_fixed_arc_distances(
        &mut self,
        fixed_arcs: &Vec<(usize, usize)>,
        costs: &Matrix<usize>,
    ) {
        fixed_arcs.iter().for_each(|fixed_arc| {
            if *self.intermediate_arc_set.get(fixed_arc.0, fixed_arc.1) {
                let _ = self.fixed_arc_distances.insert(
                    *fixed_arc,
                    *self.distance_map.get(fixed_arc.1, self.t)
                        + *costs.get(fixed_arc.0, fixed_arc.1),
                );
            }
        });
    }

    pub(crate) fn mark_arc_used(&mut self, fixed_arc: &(usize, usize)) {
        self.fixed_arc_distances.insert(*fixed_arc, usize::MAX);
    }

    pub(crate) fn closest_fixed_arc(&self) -> (bool, (usize, usize)) {
        match self
            .fixed_arc_distances
            .iter()
            .min_by_key(|entry| entry.1)
            .map(|(k, _)| k.clone())
        {
            Some(fixed_arc) => (true, fixed_arc),
            None => (false, (usize::MAX, usize::MAX)),
        }
    }
}

impl std::fmt::Display for BTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(s={}, t={}, A(s,t)={:?})",
            self.s,
            self.t,
            self.intermediate_arc_set
                .indices()
                .filter(|(s, t)| *self.intermediate_arc_set.get(*s, *t))
                .map(|(s, t)| (s, t))
                .collect::<Vec<(usize, usize)>>(),
        )
    }
}
