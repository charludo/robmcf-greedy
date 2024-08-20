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

    pub(crate) fn closest_fixed_arc(&self) -> Option<(usize, usize)> {
        match self
            .fixed_arc_distances
            .iter()
            .min_by_key(|entry| entry.1)
            .map(|(k, v)| (k.clone(), *v))
        {
            Some((fixed_arc, distance)) => match distance {
                usize::MAX => None,
                _ => Some(fixed_arc),
            },
            None => None,
        }
    }

    pub(crate) fn get_next_vertex(
        &self,
        global_waiting: &HashMap<(usize, usize), usize>,
        scenario_waiting: &HashMap<(usize, usize), usize>,
        costs: &Matrix<usize>,
    ) -> (usize, Option<(usize, usize)>) {
        let (next_vertex, fixed_arc) = match self.closest_fixed_arc() {
            None => (*self.successor_map.get(self.s, self.t), None),
            Some(fixed_arc) => {
                let relative_draw = global_waiting.get(&fixed_arc).unwrap()
                    - scenario_waiting.get(&fixed_arc).unwrap();
                let cost_via_direct_path = self.distance_map.get(self.s, self.t);
                let cost_via_fixed_arc = self.distance_map.get(self.s, fixed_arc.0)
                    + costs.get(fixed_arc.0, fixed_arc.1)
                    + self.distance_map.get(fixed_arc.1, self.t);

                if *cost_via_direct_path < cost_via_fixed_arc - relative_draw {
                    (*self.successor_map.get(self.s, self.t), None)
                } else {
                    let n_v = *self.successor_map.get(self.s, fixed_arc.0);
                    (n_v, Some(fixed_arc))
                }
            }
        };

        (next_vertex, fixed_arc)
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
