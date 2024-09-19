use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    auxiliary::{
        generate_intermediate_arc_sets, generate_supply_tokens, AuxiliaryNetwork, NetworkState,
        Scenario,
    },
    Matrix, Network, Result,
};

impl AuxiliaryNetwork {
    pub(crate) fn from_network(network: &Network) -> Result<Self> {
        let num_vertices = network.vertices.len();
        let arc_loads = Matrix::filled_with(0, num_vertices, num_vertices);

        let mut capacities = network.capacities.clone();
        for (a_0, a_1) in &network.fixed_arcs {
            capacities.set(*a_0, *a_1, usize::MAX);
        }

        // while in later iterations, capacities can differ between (s, t) pairs in tokens,
        // we can initially reuse distance and successor maps between all (s, t) pairs and
        // balances, since the arcs for the globally shortest path from s to t is guaranteed to
        // be included in the intermediate arc set of (s, t).
        let (distance_map, predecessor_map) = floyd_warshall(&capacities, &network.costs);
        let successor_map = invert_predecessors(&predecessor_map)?;

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets = generate_intermediate_arc_sets(
            &distance_map,
            &network.costs,
            &capacities,
            &network.options.delta_fn,
        );

        let scenarios: DashMap<usize, Scenario> = DashMap::new();
        let slack = network.options.slack_fn.apply(&network.balances);
        network
            .balances
            .iter()
            .enumerate()
            .for_each(|(i, balance)| {
                let network_state = NetworkState {
                    intermediate_arc_sets: arc_sets.clone(),
                    fixed_arcs: network.fixed_arcs.clone(),
                    distances: Matrix::filled_with(
                        distance_map.clone(),
                        num_vertices,
                        num_vertices,
                    ),
                    successors: Matrix::filled_with(
                        successor_map.clone(),
                        num_vertices,
                        num_vertices,
                    ),
                    capacities: capacities.clone(),
                    costs: Arc::new(network.costs.clone()),
                    arc_loads: arc_loads.clone(),
                    needs_refresh: Matrix::filled_with(true, num_vertices, num_vertices),
                    relative_draws: HashMap::new(),
                };

                let supply_tokens = generate_supply_tokens(
                    balance,
                    &network.fixed_arcs,
                    network.options.remainder_solve_method.clone(),
                    &arc_sets,
                );
                let scenario = Scenario {
                    id: i,
                    supply_tokens,
                    slack: slack[i],
                    slack_used: 0,
                    supply_remaining: balance.clone(),
                    network_state,
                };
                log::debug!("Generated {}", scenario);
                scenarios.insert(i, scenario);
            });

        Ok(AuxiliaryNetwork {
            scenarios,
            fixed_arcs: network.fixed_arcs.clone(),
        })
    }
}
