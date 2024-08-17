use std::sync::Arc;

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    matrix::Matrix,
};

use super::{
    preprocessing::{generate_b_tuples, intermediate_arc_sets},
    scenario::Scenario,
    Network,
};

#[derive(Debug)]
pub(super) struct AuxiliaryNetwork {
    pub(super) num_vertices: usize,
    pub(super) costs: Matrix<usize>,
    pub(super) scenarios: Vec<Box<Scenario>>,
    pub(super) fixed_arcs: Vec<(usize, usize)>,
    pub(super) fixed_arcs_memory: Vec<(usize, usize)>,
    pub(super) intermediate_arc_sets: Matrix<Arc<Matrix<bool>>>,
}

impl AuxiliaryNetwork {
    fn max_consistent_flows(&self) -> Vec<usize> {
        let mut max_flow_values: Vec<usize> = vec![usize::MAX; self.fixed_arcs.len()];
        self.scenarios.iter().for_each(|scenario| {
            max_flow_values
                .iter_mut()
                .enumerate()
                .for_each(|(i, f_v)| *f_v = std::cmp::max(*f_v, scenario.b_tuples_fixed[i].len()));
        });
        max_flow_values
    }

    fn create_extension_vertex(
        matrix: &Matrix<usize>,
        s: usize,
        t: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut new_row: Vec<usize> = vec![0; matrix.row_len()];
        new_row[t] = *matrix.get(s, t);

        let mut new_column = matrix.as_columns()[s].clone();
        new_column.push(0);

        (new_row, new_column)
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

        for a in network.fixed_arcs.iter() {
            let (row, col) = AuxiliaryNetwork::create_extension_vertex(&capacities, a.0, a.1);
            capacities.extend(row, col);
            capacities.set(a.0, a.1, 0);

            let (row, col) = AuxiliaryNetwork::create_extension_vertex(&costs, a.0, a.1);
            costs.extend(row, col);
            costs.set(a.0, a.1, 0);

            balances.iter_mut().for_each(|balance| {
                let (row, col) = AuxiliaryNetwork::create_extension_vertex(&balance, a.0, a.1);
                balance.extend(row, col);
                balance.set(a.0, a.1, 0);
            });

            num_vertices += 1;
            fixed_arcs.push((num_vertices, a.1));
            fixed_arcs_memory.push((num_vertices, a.0));

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
        let arc_sets = Matrix::from_elements(
            &intermediate_arc_sets(&distance_map, &capacities, |x| 2 * x)
                .elements()
                .map(|x| Arc::new(x.clone()))
                .collect(),
            num_vertices,
            num_vertices,
        ); // TODO: get d
           // from
           // somewehere...

        balances.iter().for_each(|balance| {
            let (b_tuples_free, b_tuples_fixed) =
                generate_b_tuples(&balance, &arc_sets, &fixed_arcs);
            scenarios.push(Box::new(Scenario {
                capacities: capacities.clone(),
                distance_map: distance_map.clone(),
                successor_map: successor_map.clone(),
                b_tuples_free,
                b_tuples_fixed,
            }));
        });

        AuxiliaryNetwork {
            num_vertices,
            costs,
            scenarios,
            fixed_arcs,
            fixed_arcs_memory,
            intermediate_arc_sets: arc_sets,
        }
    }
}
