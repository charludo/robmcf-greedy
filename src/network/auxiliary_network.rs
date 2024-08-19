use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    matrix::Matrix,
    network::preprocessing::create_extension_vertex,
};

use super::{
    preprocessing::{generate_b_tuples, intermediate_arc_sets},
    scenario::Scenario,
    Network,
};

#[derive(Debug)]
pub(crate) struct AuxiliaryNetwork {
    pub(crate) num_vertices: usize,
    pub(crate) costs: Matrix<usize>,
    pub(crate) scenarios: Vec<Box<Scenario>>,
    pub(crate) fixed_arcs: Vec<(usize, usize)>,
    pub(crate) fixed_arcs_memory: Vec<(usize, usize)>,
    pub(crate) intermediate_arc_sets: Matrix<Arc<Matrix<bool>>>,
    pub(crate) arc_loads: Vec<Matrix<usize>>,
}

impl AuxiliaryNetwork {
    pub(crate) fn max_consistent_flows(&self) -> HashMap<(usize, usize), usize> {
        let mut max_flow_values: HashMap<(usize, usize), usize> = HashMap::new();
        self.fixed_arcs.iter().for_each(|fixed_arc| {
            self.scenarios.iter().for_each(|scenario| {
                let _ = max_flow_values.insert(
                    *fixed_arc,
                    *std::cmp::max(
                        max_flow_values.get(fixed_arc).unwrap_or(&0),
                        &scenario.waiting_at(fixed_arc),
                    ),
                );
            });
        });
        max_flow_values
    }

    pub(crate) fn waiting(&self) -> HashMap<(usize, usize), usize> {
        let mut wait_map: HashMap<(usize, usize), usize> = HashMap::new();
        self.fixed_arcs.iter().for_each(|fixed_arc| {
            wait_map.insert(*fixed_arc, self.waiting_at(fixed_arc));
        });
        wait_map
    }

    pub(crate) fn waiting_at(&self, fixed_arc: &(usize, usize)) -> usize {
        self.scenarios.iter().map(|s| s.waiting_at(fixed_arc)).sum()
    }

    pub(crate) fn exists_free_supply(&self) -> bool {
        self.scenarios
            .iter()
            .map(|s| s.b_tuples_free.len())
            .sum::<usize>()
            != 0
    }
}

impl From<&Network> for AuxiliaryNetwork {
    fn from(network: &Network) -> Self {
        let mut num_vertices = network.vertices.len();
        let mut fixed_arcs: Vec<(usize, usize)> = vec![];
        let mut fixed_arcs_memory: Vec<(usize, usize)> = vec![];
        let mut costs = network.costs.clone();
        let mut capacities = network.capacities.clone();
        let mut balances = network.balances.clone();
        let mut scenarios: Vec<Box<Scenario>> = vec![];
        let arc_loads =
            vec![Matrix::filled_with(0, num_vertices, num_vertices); network.balances.len()];

        for a in network.fixed_arcs.iter() {
            let (row, col) = create_extension_vertex(&capacities, a.0, a.1);
            capacities.extend(&row, &col);
            capacities.set(a.0, a.1, 0);

            let (row, col) = create_extension_vertex(&costs, a.0, a.1);
            costs.extend(&row, &col);
            costs.set(a.0, a.1, 0);

            balances.iter_mut().for_each(|balance| {
                let (row, col) = create_extension_vertex(&balance, a.0, a.1);
                balance.extend(&row, &col);
                balance.set(a.0, a.1, 0);
            });

            fixed_arcs.push((num_vertices, a.1));
            fixed_arcs_memory.push((num_vertices, a.0));
            num_vertices += 1;

            log::debug!(
                "Extended the network with an auxiliary fixed arc ({}->{}) replacing ({}->{})",
                num_vertices,
                a.1,
                a.0,
                a.1
            );
        }

        // while in later iterations, capacities can differ between (s, t) pairs in BTuples,
        // we can initially re-use distance and successor maps between all (s, t) pairs and
        // balances, since the arcs for the globally shortest path from s to t is guaranteed to
        // be included in in the intermediate arc set of (s, t).
        let (distance_map, predecessor_map) = floyd_warshall(&capacities, &costs);
        let successor_map = invert_predecessors(&predecessor_map);

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets = intermediate_arc_sets(&distance_map, &costs, &capacities, |x| 2 * x); // TODO: get d
                                                                                             // from
                                                                                             // somewehere...
        balances.iter().for_each(|balance| {
            let (b_tuples_free, b_tuples_fixed) =
                generate_b_tuples(&balance, &arc_sets, &fixed_arcs);
            let scenario = Scenario {
                capacities: capacities.clone(),
                distance_map: distance_map.clone(),
                successor_map: successor_map.clone(),
                b_tuples_free,
                b_tuples_fixed,
            };
            log::debug!("Generated the following scenario:\n{}", scenario);
            scenarios.push(Box::new(scenario));
        });

        AuxiliaryNetwork {
            num_vertices,
            costs,
            scenarios,
            fixed_arcs,
            fixed_arcs_memory,
            intermediate_arc_sets: arc_sets,
            arc_loads,
        }
    }
}
