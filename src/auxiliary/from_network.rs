use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::floyd_warshall,
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
        let (distance_map, _) = floyd_warshall(&capacities, &network.costs);

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets = generate_intermediate_arc_sets(
            &distance_map,
            &network.costs,
            &capacities,
            &network.options.delta_fn,
        );

        let scenarios: DashMap<usize, Scenario> = DashMap::new();
        for (i, balance) in network.balances.iter().enumerate() {
            let network_state = NetworkState {
                scenario_id: i,
                fixed_arcs: network.fixed_arcs.clone(),
                capacities: capacities.clone(),
                costs: Arc::new(network.costs.clone()),
                arc_loads: arc_loads.clone(),
                relative_draws: HashMap::new(),
            };

            let mut supply_tokens = generate_supply_tokens(
                balance,
                &network.fixed_arcs,
                network.options.remainder_solve_method.clone(),
                &arc_sets,
                &capacities,
                &network.costs,
            )?;
            supply_tokens.sort_by_key(|token| *distance_map.get(token.s, token.t));

            let scenario = Scenario {
                id: i,
                supply_tokens,
                supply_remaining: balance.clone(),
                network_state,
            };
            log::debug!("Generated {}", scenario);
            scenarios.insert(i, scenario);
        }

        Ok(AuxiliaryNetwork {
            scenarios,
            fixed_arcs: network.fixed_arcs.clone(),
        })
    }
}
