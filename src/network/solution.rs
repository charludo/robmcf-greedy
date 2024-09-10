use colored::{ColoredString, Colorize};
use std::fmt::Display;

use crate::matrix::Matrix;

use super::Network;

#[derive(Debug, Clone)]
pub(crate) struct Solution {
    pub(crate) slack: Vec<usize>,
    pub(crate) slack_used: Vec<usize>,
    pub(crate) costs: Vec<usize>,
    pub(crate) network_cost: usize,
    pub(crate) arc_loads: Vec<Matrix<usize>>,
    pub(crate) supply_total: Vec<usize>,
    pub(crate) supply_remaining: Vec<Matrix<usize>>,
    arc_loads_repr: Vec<Matrix<ColoredString>>,
}

impl From<&Network> for Solution {
    fn from(network: &Network) -> Self {
        let mut arc_loads: Vec<Matrix<usize>> = Vec::new();
        let mut arc_loads_repr: Vec<Matrix<ColoredString>> = Vec::new();
        let mut slack: Vec<usize> = Vec::new();
        let mut slack_used: Vec<usize> = Vec::new();
        let mut costs: Vec<usize> = Vec::new();
        let mut supply_total: Vec<usize> = Vec::new();
        let mut supply_remaining: Vec<Matrix<usize>> = Vec::new();

        if let Some(auxiliary_network) = &network.auxiliary_network {
            auxiliary_network.scenarios.iter().for_each(|scenario| {
                let mut scenario_arc_loads = scenario.network_state.arc_loads.clone();
                auxiliary_network.fixed_arcs.iter().for_each(|fixed_arc| {
                    let original_arc = auxiliary_network.fixed_arcs_memory.get(fixed_arc).unwrap();
                    scenario_arc_loads.set(
                        original_arc.0,
                        original_arc.1,
                        *scenario_arc_loads.get(*fixed_arc, original_arc.1),
                    );
                });
                scenario_arc_loads.shrink(auxiliary_network.fixed_arcs.len());
                slack.push(scenario.slack);
                slack_used.push(scenario.slack_used);
                costs.push(scenario_arc_loads.hadamard_product(&network.costs).sum());
                let mut scenario_arc_loads_str: Matrix<ColoredString> = Matrix::from_elements(
                    scenario_arc_loads
                        .elements()
                        .map(|x| x.to_string().white())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    scenario_arc_loads.num_rows(),
                    scenario_arc_loads.num_columns(),
                );
                network.fixed_arcs.iter().for_each(|(a0, a1)| {
                    scenario_arc_loads_str.set(
                        *a0,
                        *a1,
                        scenario_arc_loads_str.get(*a0, *a1).clone().green(),
                    );
                });
                arc_loads.push(scenario_arc_loads);
                arc_loads_repr.push(scenario_arc_loads_str);

                let mut remaining_supply = scenario.remaining_supply.clone();
                remaining_supply.shrink(auxiliary_network.fixed_arcs.len());
                supply_total.push(network.balances[scenario.id].sum());
                supply_remaining.push(remaining_supply);
            });
        };
        Solution {
            arc_loads,
            arc_loads_repr,
            network_cost: network.options.cost_fn.apply(&costs),
            costs,
            slack,
            slack_used,
            supply_total,
            supply_remaining,
        }
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The following arc loads constitute the solution:\n{}\nThe network cost is {}.",
            (0..self.arc_loads.len())
                .map(|i| format!(
                    "Scenario {}, with cost {} and {}/{} slack used in delivery of {}/{} supply units:\n{}",
                    i,
                    self.costs[i],
                    self.slack_used[i],
                    self.slack[i] + self.slack_used[i],
                    self.supply_total[i] - self.supply_remaining[i].sum(),
                    self.supply_total[i],
                    self.arc_loads_repr[i]
                ))
                .collect::<Vec<String>>()
                .join("\n"),
            self.network_cost
        )
    }
}
