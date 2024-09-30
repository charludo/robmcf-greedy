use std::cmp::Ordering;

use colored::{Color, ColoredString, Colorize};

use crate::{options::CostFunction, Matrix};

#[derive(Debug, Clone)]
pub(crate) struct ScenarioSolution {
    pub(crate) id: usize,
    pub(crate) slack: usize,
    pub(crate) supply_remaining: Matrix<usize>,
    pub(crate) arc_loads: Matrix<usize>,
}

impl ScenarioSolution {
    pub(crate) fn new(id: usize, supply: &Matrix<usize>) -> Self {
        ScenarioSolution {
            id,
            slack: 0,
            supply_remaining: supply.clone(),
            arc_loads: Matrix::filled_with(0, supply.num_rows(), supply.num_columns()),
        }
    }

    pub(crate) fn cost(&self, cost_matrix: &Matrix<usize>) -> usize {
        self.arc_loads.hadamard_product(cost_matrix).sum()
    }

    pub(crate) fn supply_delivered(&self, supply_total: usize) -> usize {
        supply_total - self.supply_remaining.sum()
    }
}

pub trait Solution {
    fn cost(&self, cost_matrix: &Matrix<usize>, cost_fn: &CostFunction) -> usize;
    fn consistent_flows(&self, fixed_arcs: &[(usize, usize)]) -> Matrix<i64>;
    fn consistent_flows_colorized(
        &self,
        fixed_arcs: &[(usize, usize)],
        color: Color,
    ) -> Matrix<ColoredString>;
    fn highlight_difference_to(
        &self,
        other: &[ScenarioSolution],
        fixed_arcs: &[(usize, usize)],
    ) -> Matrix<ColoredString>;
}

impl Solution for [ScenarioSolution] {
    fn cost(&self, cost_matrix: &Matrix<usize>, cost_fn: &CostFunction) -> usize {
        cost_fn.apply(
            self.iter()
                .map(|s| s.cost(cost_matrix))
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }

    fn consistent_flows(&self, fixed_arcs: &[(usize, usize)]) -> Matrix<i64> {
        let mut fixed_arc_loads = Matrix::filled_with(
            i64::MAX,
            self[0].arc_loads.num_rows(),
            self[0].arc_loads.num_columns(),
        );
        for (s, t) in fixed_arcs.iter() {
            let min_load = *self
                .iter()
                .map(|scenario| scenario.arc_loads.get(*s, *t))
                .min()
                .unwrap_or(&0) as i64;
            fixed_arc_loads.set(*s, *t, min_load);
        }
        fixed_arc_loads
    }

    fn consistent_flows_colorized(
        &self,
        fixed_arcs: &[(usize, usize)],
        color: Color,
    ) -> Matrix<ColoredString> {
        let fixed_arc_loads = self.consistent_flows(fixed_arcs);
        Matrix::from_elements(
            fixed_arc_loads
                .elements()
                .map(|e| {
                    if *e == i64::MAX {
                        " ".to_string()
                    } else {
                        e.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .as_slice(),
            fixed_arc_loads.num_rows(),
            fixed_arc_loads.num_columns(),
        )
        .highlight(fixed_arcs, color)
    }

    fn highlight_difference_to(
        &self,
        other: &[ScenarioSolution],
        fixed_arcs: &[(usize, usize)],
    ) -> Matrix<ColoredString> {
        let self_flows = self.consistent_flows(fixed_arcs);
        let other_flows = other.consistent_flows(fixed_arcs);
        Matrix::from_elements(
            self_flows
                .indices()
                .map(|(s, t)| {
                    let first = *self_flows.get(s, t);
                    let second = *other_flows.get(s, t);
                    if first == i64::MAX || second == i64::MAX {
                        " ".to_string().color(Color::White)
                    } else {
                        let diff = first - second;
                        match diff.cmp(&0) {
                            Ordering::Equal => diff.to_string().color(Color::Blue),
                            Ordering::Greater => diff.to_string().color(Color::Green),
                            Ordering::Less => diff.to_string().color(Color::Red),
                        }
                    }
                })
                .collect::<Vec<_>>()
                .as_slice(),
            self_flows.num_rows(),
            self_flows.num_columns(),
        )
    }
}
