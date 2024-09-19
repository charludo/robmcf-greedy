mod from_network;
mod network_state;
mod preprocessing;
mod scenario;
mod supply_token;

use dashmap::DashMap;
use std::collections::HashMap;

pub(super) use network_state::NetworkState;
pub(super) use preprocessing::{generate_intermediate_arc_sets, generate_supply_tokens};
pub(crate) use scenario::Scenario;

#[derive(Debug, Clone)]
pub(crate) struct AuxiliaryNetwork {
    pub(crate) fixed_arcs: Vec<(usize, usize)>,
    pub(crate) scenarios: DashMap<usize, Scenario>,
}

impl AuxiliaryNetwork {
    pub(crate) fn snapshot_fixed_arc_loads(&self) -> HashMap<(usize, usize), Vec<i64>> {
        let mut snapshot = HashMap::new();
        for (a_0, a_1) in &self.fixed_arcs {
            let _ = snapshot.insert(
                (*a_0, *a_1),
                self.scenarios
                    .iter()
                    .map(|scenario| *scenario.network_state.arc_loads.get(*a_0, *a_1) as i64)
                    .collect::<Vec<_>>(),
            );
        }
        snapshot
    }

    pub(crate) fn exists_supply(&self) -> bool {
        self.scenarios
            .iter()
            .map(|s| s.supply_tokens.len())
            .sum::<usize>()
            != 0
    }
}
