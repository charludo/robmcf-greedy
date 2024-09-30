use serde::Serialize;

use crate::{network::Solution, CostFunction};

use super::Network;

#[derive(Serialize, Debug)]
pub(super) struct NetworkData {
    vertex_count: usize,
    station_density: f32,

    arc_density: f32,
    num_fixed_arcs: usize,
    robustness_gain: Option<i64>,

    num_scenarios: usize,
    supply_density_min: f32,
    supply_density_max: f32,
    supply_density_avg: f32,
    supply_min: usize,
    supply_avg: usize,
    supply_max: usize,

    slack_min: usize,
    slack_avg: usize,
    slack_max: usize,

    lower_bound_max: Option<usize>,
    lower_bound_mean: Option<usize>,
    lower_bound_median: Option<usize>,
    cost_max: usize,
    cost_mean: usize,
    cost_median: usize,

    draw_fn: String,
    delta_fn: String,
    remainder_fn: String,

    time_preprocess: Option<usize>,
    time_solve: Option<usize>,
}

impl NetworkData {
    pub(super) fn from_network(
        network: &Network,
        time_preprocess: Option<usize>,
        time_solve: Option<usize>,
    ) -> Self {
        NetworkData {
            vertex_count: network.vertices.len(),
            station_density: network.vertices.iter().filter(|v| v.is_station).count() as f32
                / network.vertices.len() as f32,

            arc_density: network.capacities.elements().filter(|e| **e > 0).count() as f32
                / network.vertices.len().pow(2) as f32,
            num_fixed_arcs: network.fixed_arcs.len(),
            robustness_gain: match (&network.baseline, &network.solutions) {
                (Some(baseline), Some(solutions)) => Some(
                    solutions
                        .consistent_flows(&network.fixed_arcs)
                        .subtract(&baseline.consistent_flows(&network.fixed_arcs))
                        .sum(),
                ),
                _ => None,
            },

            num_scenarios: network.balances.len(),
            supply_density_min: network
                .balances
                .iter()
                .map(|b| {
                    b.as_rows()
                        .iter()
                        .map(|r| r.iter().filter(|e| **e > 0).count() as f32)
                        .sum::<f32>()
                        / network.vertices.len() as f32
                })
                .fold(f32::INFINITY, |a, b| a.min(b)),
            supply_density_avg: network
                .balances
                .iter()
                .map(|b| {
                    b.as_rows()
                        .iter()
                        .map(|r| r.iter().filter(|e| **e > 0).count() as f32)
                        .sum::<f32>()
                        / network.vertices.len() as f32
                })
                .sum::<f32>()
                / network.balances.len() as f32,
            supply_density_max: network
                .balances
                .iter()
                .map(|b| {
                    b.as_rows()
                        .iter()
                        .map(|r| r.iter().filter(|e| **e > 0).count() as f32)
                        .sum::<f32>()
                        / network.vertices.len() as f32
                })
                .fold(f32::NEG_INFINITY, |a, b| a.max(b)),

            supply_min: network
                .balances
                .iter()
                .map(|b| b.sum())
                .min()
                .unwrap_or_default(),

            supply_avg: network.balances.iter().map(|b| b.sum()).sum::<usize>()
                / network.balances.len(),

            supply_max: network
                .balances
                .iter()
                .map(|b| b.sum())
                .max()
                .unwrap_or_default(),

            slack_min: network
                .solutions
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|s| s.slack)
                .min()
                .unwrap_or_default(),
            slack_avg: network
                .solutions
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|s| s.slack)
                .sum::<usize>()
                / network.balances.len(),
            slack_max: network
                .solutions
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|s| s.slack)
                .max()
                .unwrap_or_default(),

            lower_bound_max: network
                .baseline
                .as_ref()
                .map(|baseline| baseline.cost(&network.costs, &CostFunction::Max)),
            lower_bound_mean: network
                .baseline
                .as_ref()
                .map(|baseline| baseline.cost(&network.costs, &CostFunction::Mean)),
            lower_bound_median: network
                .baseline
                .as_ref()
                .map(|baseline| baseline.cost(&network.costs, &CostFunction::Median)),

            cost_max: match &network.solutions {
                Some(solutions) => solutions.cost(&network.costs, &CostFunction::Max),
                None => 0,
            },
            cost_mean: match &network.solutions {
                Some(solutions) => solutions.cost(&network.costs, &CostFunction::Mean),
                None => 0,
            },
            cost_median: match &network.solutions {
                Some(solutions) => solutions.cost(&network.costs, &CostFunction::Median),
                None => 0,
            },

            draw_fn: network.options.relative_draw_fn.to_string(),
            delta_fn: network.options.delta_fn.to_string(),
            remainder_fn: network.options.remainder_solve_method.shorthand(),

            time_preprocess,
            time_solve,
        }
    }
}
