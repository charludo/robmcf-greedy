use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    matrix::Matrix,
    network::preprocessing::create_extension_vertex,
};

use super::{
    network_state::NetworkState,
    preprocessing::{generate_b_tuples, generate_intermediate_arc_sets},
    scenario::Scenario,
    Network,
};

#[derive(Debug)]
pub(crate) struct AuxiliaryNetwork {
    pub(crate) costs: Matrix<usize>,
    pub(crate) fixed_arcs: Vec<usize>,
    pub(crate) fixed_arcs_memory: HashMap<usize, usize>,

    pub(crate) scenarios: Vec<Box<Scenario>>,
    pub(crate) network_states: Vec<Box<NetworkState>>,
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
}

impl From<&Network> for AuxiliaryNetwork {
    fn from(network: &Network) -> Self {
        let mut num_vertices = network.vertices.len();
        let mut fixed_arcs: Vec<usize> = vec![];
        let mut fixed_arcs_memory: HashMap<usize, usize> = HashMap::new();
        let mut costs = network.costs.clone();
        let mut capacities = network.capacities.clone();
        let mut balances = network.balances.clone();
        let mut scenarios: Vec<Box<Scenario>> = vec![];
        let mut network_states: Vec<Box<NetworkState>> = vec![];

        for a in network.fixed_arcs.iter() {
            let (row, col) = create_extension_vertex(&capacities, a.0, a.1);
            capacities.extend(&row, &col);
            capacities.set(a.0, a.1, 0);
            capacities.set(num_vertices, a.1, usize::MAX); // fixed arcs have infinite capacity
            capacities.set(a.0, num_vertices, usize::MAX); // additionally, add an infinite-capacity
                                                           // arc for flow from a.0 wanting to pass along the
                                                           // new fixed arc (set to cost 0 below)

            let (row, col) = create_extension_vertex(&costs, a.0, a.1);
            costs.extend(&row, &col);
            costs.set(a.0, a.1, 0);
            costs.set(a.0, num_vertices, 0); // prevents calculating cost twice
                                             // (old and new fixed arc)

            balances.iter_mut().for_each(|balance| {
                balance.extend(&vec![0; num_vertices], &vec![0; num_vertices + 1]);
            });

            fixed_arcs.push(num_vertices);
            fixed_arcs_memory.insert(num_vertices, a.0);

            log::debug!(
                "Extended the network with an auxiliary fixed arc ({}->{}) replacing ({}->{})",
                num_vertices,
                a.1,
                a.0,
                a.1
            );
            log::trace!("Capacities now look like this:\n{}", capacities);
            log::trace!("Costs now look like this:\n{}", costs);
            log::trace!(
                "Balances now look like this:\n{}",
                balances
                    .iter()
                    .map(|b| format!("{}", b))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            num_vertices += 1;
        }

        let arc_loads = Matrix::filled_with(0, num_vertices, num_vertices);

        // while in later iterations, capacities can differ between (s, t) pairs in BTuples,
        // we can initially re-use distance and successor maps between all (s, t) pairs and
        // balances, since the arcs for the globally shortest path from s to t is guaranteed to
        // be included in in the intermediate arc set of (s, t).
        let (distance_map, predecessor_map) = floyd_warshall(&capacities, &costs);
        let successor_map = invert_predecessors(&predecessor_map);

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets =
            generate_intermediate_arc_sets(&distance_map, &costs, &capacities, |x| 2 * x); // TODO: get d
                                                                                           // from
                                                                                           // somewehere...
        balances.iter().enumerate().for_each(|(i, balance)| {
            let (b_tuples_free, b_tuples_fixed) = generate_b_tuples(&balance);
            let scenario = Scenario {
                id: i,
                b_tuples_free,
                b_tuples_fixed,
            };
            log::debug!("Generated {}", scenario);
            scenarios.push(Box::new(scenario));

            let network_state = NetworkState {
                intermediate_arc_sets: arc_sets.clone(),
                fixed_arcs: Matrix::from_elements(
                    &vec![fixed_arcs.clone(); num_vertices],
                    num_vertices,
                    num_vertices,
                ),
                distances: Matrix::from_elements(
                    &vec![distance_map.clone(); num_vertices],
                    num_vertices,
                    num_vertices,
                ),
                successors: Matrix::from_elements(
                    &vec![successor_map.clone(); num_vertices],
                    num_vertices,
                    num_vertices,
                ),
                capacities: capacities.clone(),
                costs: Arc::new(costs.clone()),
                arc_loads: arc_loads.clone(),
                slack: 0,
            };
            network_states.push(Box::new(network_state));
        });

        AuxiliaryNetwork {
            costs,
            scenarios,
            network_states,
            fixed_arcs,
            fixed_arcs_memory,
        }
    }
}
