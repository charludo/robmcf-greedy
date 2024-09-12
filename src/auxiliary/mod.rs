mod b_tuple;
mod from_network;
mod network_state;
mod preprocessing;
mod scenario;

use dashmap::DashMap;
use std::collections::HashMap;

pub(crate) use scenario::Scenario;

use crate::{Result, SolverError};

#[derive(Debug, Clone)]
pub(crate) struct AuxiliaryNetwork {
    pub(crate) fixed_arcs: Vec<usize>,
    pub(crate) fixed_arcs_memory: HashMap<usize, (usize, usize)>,
    pub(crate) scenarios: DashMap<usize, Scenario>,
}

impl AuxiliaryNetwork {
    pub(crate) fn max_consistent_flows(&self) -> HashMap<usize, usize> {
        let mut max_flow_values: HashMap<usize, usize> = HashMap::new();
        self.scenarios.iter().for_each(|scenario| {
            self.fixed_arcs.iter().for_each(|fixed_arc| {
                let _ = max_flow_values.insert(
                    *fixed_arc,
                    *std::cmp::min(
                        max_flow_values.get(fixed_arc).unwrap_or(&usize::MAX),
                        &scenario.waiting_at(*fixed_arc),
                    ),
                );
            });
        });
        max_flow_values
    }

    pub(crate) fn waiting(&self) -> HashMap<usize, usize> {
        let mut wait_map: HashMap<usize, usize> = HashMap::new();
        self.fixed_arcs.iter().for_each(|fixed_arc| {
            wait_map.insert(*fixed_arc, self.waiting_at(*fixed_arc));
        });
        wait_map
    }

    pub(crate) fn waiting_at(&self, fixed_arc: usize) -> usize {
        self.scenarios.iter().map(|s| s.waiting_at(fixed_arc)).sum()
    }

    pub(crate) fn exists_free_supply(&self) -> bool {
        self.scenarios
            .iter()
            .map(|s| s.b_tuples_free.len())
            .sum::<usize>()
            != 0
    }

    pub(crate) fn exists_fixed_supply(&self) -> bool {
        self.scenarios
            .iter()
            .map(|s| s.b_tuples_fixed.values().map(|v| v.len()).sum::<usize>())
            .sum::<usize>()
            != 0
    }

    pub(crate) fn exists_supply(&self) -> bool {
        self.exists_free_supply() || self.exists_fixed_supply()
    }

    pub(crate) fn fixed_arc_repr(&self, fixed_vertex: usize) -> String {
        let fixed_arc = self.fixed_arcs_memory.get(&fixed_vertex);
        match fixed_arc {
            Some(arc) => format!("({}->{})", arc.0, arc.1),
            None => format!("unknown fixed arc eminating from {}", fixed_vertex),
        }
    }

    pub(crate) fn get_fixed_arc_terminal(&self, fixed_vertex: usize) -> Result<usize> {
        let fixed_arc = self.fixed_arcs_memory.get(&fixed_vertex);
        match fixed_arc {
            Some(arc) => Ok(arc.1),
            None => Err(SolverError::FixedArcMemoryCorruptError),
        }
    }
}
