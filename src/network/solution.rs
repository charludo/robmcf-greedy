use std::collections::HashMap;

use colored::{ColoredString, Colorize};

use crate::matrix::Matrix;

#[derive(Debug, Clone)]
pub(crate) struct ScenarioSolution {
    pub(crate) id: usize,

    pub(crate) slack_total: usize,
    pub(crate) slack_remaining: usize,

    pub(crate) supply_remaining: Matrix<usize>,
    pub(crate) arc_loads: Matrix<usize>,
}

impl ScenarioSolution {
    pub(crate) fn new(id: usize, supply: &Matrix<usize>) -> Self {
        ScenarioSolution {
            id,
            slack_total: 0,
            slack_remaining: 0,
            supply_remaining: supply.clone(),
            arc_loads: Matrix::filled_with(0, supply.num_rows(), supply.num_columns()),
        }
    }

    pub(crate) fn cost(&self, cost_matrix: &Matrix<usize>) -> usize {
        self.arc_loads.hadamard_product(cost_matrix).sum()
    }

    pub(crate) fn arc_loads_colorized(
        &self,
        fixed_arcs: &[(usize, usize)],
    ) -> Matrix<ColoredString> {
        let mut arc_loads_colorized: Matrix<ColoredString> = Matrix::from_elements(
            self.arc_loads
                .elements()
                .map(|x| x.to_string().white())
                .collect::<Vec<_>>()
                .as_slice(),
            self.arc_loads.num_rows(),
            self.arc_loads.num_columns(),
        );
        fixed_arcs.iter().for_each(|(s, t)| {
            arc_loads_colorized.set(*s, *t, arc_loads_colorized.get(*s, *t).clone().blue());
        });
        arc_loads_colorized
    }

    pub(crate) fn supply_delivered(&self, supply_total: usize) -> usize {
        supply_total - self.supply_remaining.sum()
    }

    pub(crate) fn slack_used(&self) -> usize {
        self.slack_total - self.slack_remaining
    }

    pub(crate) fn supply_from_auxiliary(
        supply: &Matrix<usize>,
        num_fixed_arcs: usize,
    ) -> Matrix<usize> {
        let mut supply = supply.clone();
        supply.shrink(num_fixed_arcs);
        supply
    }

    pub(crate) fn arc_loads_from_auxiliary(
        arc_loads: &Matrix<usize>,
        fixed_arcs: &[usize],
        fixed_arcs_memory: &HashMap<usize, (usize, usize)>,
    ) -> Matrix<usize> {
        let mut arc_loads = arc_loads.clone();
        fixed_arcs.iter().for_each(|fixed_arc| {
            let original_arc = fixed_arcs_memory.get(fixed_arc).unwrap();
            arc_loads.set(
                original_arc.0,
                original_arc.1,
                *arc_loads.get(*fixed_arc, original_arc.1),
            );
        });
        arc_loads.shrink(fixed_arcs.len());
        arc_loads
    }
}
