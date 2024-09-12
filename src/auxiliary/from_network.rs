use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    auxiliary::{
        generate_b_tuples, generate_intermediate_arc_sets, AuxiliaryNetwork, NetworkState, Scenario,
    },
    Matrix, Network, Result,
};

impl AuxiliaryNetwork {
    pub(crate) fn from_network(network: &Network) -> Result<Self> {
        let mut num_vertices = network.vertices.len();
        let mut fixed_arcs: Vec<usize> = vec![];
        let mut fixed_arcs_memory: HashMap<usize, (usize, usize)> = HashMap::new();
        let mut costs = network.costs.clone();
        let mut capacities = network.capacities.clone();
        let mut balances = network.balances.clone();
        let scenarios: DashMap<usize, Scenario> = DashMap::new();

        for a in network.fixed_arcs.iter() {
            capacities.extend(&vec![0; num_vertices], &vec![0; num_vertices + 1]);
            capacities.set(a.0, a.1, 0);
            capacities.set(num_vertices, a.1, usize::MAX); // fixed arcs have infinite capacity
            capacities.set(a.0, num_vertices, usize::MAX); // additionally, add an infinite-capacity
                                                           // arc for flow from a.0 wanting to pass along the
                                                           // new fixed arc (set to cost 0 below)

            costs.extend(&vec![0; num_vertices], &vec![0; num_vertices + 1]);
            costs.set(num_vertices, a.1, *costs.get(a.0, a.1));
            costs.set(a.0, num_vertices, 0); // prevents calculating cost twice
                                             // (old and new fixed arc)

            balances.iter_mut().for_each(|balance| {
                balance.extend(&vec![0; num_vertices], &vec![0; num_vertices + 1]);
            });

            fixed_arcs.push(num_vertices);
            fixed_arcs_memory.insert(num_vertices, *a);

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
        // we can initially reuse distance and successor maps between all (s, t) pairs and
        // balances, since the arcs for the globally shortest path from s to t is guaranteed to
        // be included in in the intermediate arc set of (s, t).
        let (distance_map, predecessor_map) = floyd_warshall(&capacities, &costs);
        let successor_map = invert_predecessors(&predecessor_map)?;

        // intermediate arc sets only need to be computed once. Their sole purpose is to act as a
        // mask on capacities when Floyd-Warshall is refreshed in the greedy iterations.
        let arc_sets = generate_intermediate_arc_sets(
            &distance_map,
            &costs,
            &capacities,
            &network.options.delta_fn,
        );

        let slack = network.options.slack_fn.apply(&balances);
        balances.iter().enumerate().for_each(|(i, balance)| {
            let network_state = NetworkState {
                intermediate_arc_sets: arc_sets.clone(),
                fixed_arcs: Matrix::filled_with(fixed_arcs.clone(), num_vertices, num_vertices),
                distances: Matrix::filled_with(distance_map.clone(), num_vertices, num_vertices),
                successors: Matrix::filled_with(successor_map.clone(), num_vertices, num_vertices),
                capacities: capacities.clone(),
                costs: Arc::new(costs.clone()),
                arc_loads: arc_loads.clone(),
                needs_refresh: Matrix::filled_with(true, num_vertices, num_vertices),
                relative_draws: HashMap::new(),
            };

            let (b_tuples_free, b_tuples_fixed) = generate_b_tuples(
                balance,
                network.options.remainder_solve_method.clone(),
                network.fixed_arcs.len(),
                &arc_sets,
            );
            let scenario = Scenario {
                id: i,
                b_tuples_free,
                b_tuples_fixed,
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
            fixed_arcs,
            fixed_arcs_memory,
        })
    }
}
