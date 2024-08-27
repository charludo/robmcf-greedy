use colored::Colorize;
use std::fmt::Display;

use crate::matrix::Matrix;

use super::Network;

#[derive(Debug)]
pub(crate) struct Solution {
    pub(crate) arc_loads: Vec<Matrix<usize>>,
    pub(crate) slack: Vec<usize>,
    pub(crate) costs: Vec<usize>,
    arc_loads_repr: Vec<Matrix<String>>,
}

impl From<&Network> for Solution {
    fn from(network: &Network) -> Self {
        let mut arc_loads: Vec<Matrix<usize>> = Vec::new();
        let mut arc_loads_repr: Vec<Matrix<String>> = Vec::new();
        let mut slack: Vec<usize> = Vec::new();
        let mut costs: Vec<usize> = Vec::new();
        if let Some(auxiliary_network) = &network.auxiliary_network {
            auxiliary_network.scenarios.iter().for_each(|scenario| {
                let mut scenario_arc_loads = scenario.network_state.arc_loads.clone();
                auxiliary_network.fixed_arcs.iter().for_each(|fixed_arc| {
                    let original_arc = auxiliary_network.fixed_arcs_memory.get(&fixed_arc).unwrap();
                    scenario_arc_loads.set(
                        original_arc.0,
                        original_arc.1,
                        *scenario_arc_loads.get(*fixed_arc, original_arc.1),
                    );
                });
                scenario_arc_loads.shrink(auxiliary_network.fixed_arcs.len());
                slack.push(scenario.slack);
                costs.push(scenario_arc_loads.hadamard_product(&network.costs).sum());
                let mut scenario_arc_loads_str = Matrix::from_elements(
                    &scenario_arc_loads
                        .elements()
                        .map(|x| x.to_string())
                        .collect(),
                    scenario_arc_loads.num_rows(),
                    scenario_arc_loads.num_columns(),
                );
                network.fixed_arcs.iter().for_each(|(a0, a1)| {
                    scenario_arc_loads_str.set(
                        *a0,
                        *a1,
                        scenario_arc_loads_str.get(*a0, *a1).green().to_string(),
                    );
                });
                arc_loads.push(scenario_arc_loads);
                arc_loads_repr.push(scenario_arc_loads_str);
            });
        };
        Solution {
            arc_loads,
            arc_loads_repr,
            costs,
            slack,
        }
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The following arc loads constitute the solution:\n{}",
            (0..self.arc_loads.len())
                .map(|i| format!(
                    "Scenario {}, with cost {} and {} slack:\n{}",
                    i, self.costs[i], self.slack[i], self.arc_loads_repr[i]
                ))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
