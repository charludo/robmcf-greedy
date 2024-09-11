use std::fmt::Display;

use colored::ColoredString;

use crate::{matrix::Matrix, options::CostFunction};

use super::ScenarioSolution;

#[derive(Debug, Clone)]
pub(crate) struct Baseline {
    costs: usize,
    fixed_arc_loads: Matrix<ColoredString>,
}

impl Baseline {
    pub(crate) fn from_solutions(
        solutions: &[ScenarioSolution],
        fixed_arcs: &[(usize, usize)],
        cost_matrix: &Matrix<usize>,
        cost_fn: &CostFunction,
    ) -> Self {
        let costs = cost_fn.apply(
            solutions
                .iter()
                .map(|s| s.cost(cost_matrix))
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let mut fixed_arc_loads = Matrix::filled_with(
            " ".to_string(),
            cost_matrix.num_rows(),
            cost_matrix.num_columns(),
        );
        for (s, t) in fixed_arcs.iter() {
            let min_load = solutions
                .iter()
                .map(|scenario| scenario.arc_loads.get(*s, *t))
                .min()
                .unwrap_or(&0)
                .to_string();
            fixed_arc_loads.set(*s, *t, min_load);
        }

        Baseline {
            costs,
            fixed_arc_loads: fixed_arc_loads.highlight(fixed_arcs, colored::Color::Blue),
        }
    }
}

impl Display for Baseline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The lower bound on network cost is {}. The following consistent flow values are achieved without consistent flow constraints:\n{}", self.costs, self.fixed_arc_loads)
    }
}
