use std::fmt::Display;

use colored::Color;

use crate::network::{Network, Solution};

impl Network {
    fn display_solutions(&self) -> String {
        let Some(solutions) = &self.solutions else {
            return "No solutions have been found yet.".to_string();
        };
        format!(
            "The following arc loads constitute the solution:\n{}\nThe network cost is {}.",
            solutions.iter()
                .map(|solution| format!(
                    "Scenario {}, with cost {} and {} slack used (target: ≤ {}) in delivery of {}/{} supply units:\n{}",
                    solution.id,
                    solution.cost(&self.costs),
                    solution.slack,
                    self.options.slack_fn.apply(&self.balances)[solution.id],
                    solution.supply_delivered(self.balances[solution.id].sum()),
                    self.balances[solution.id].sum(),
                    solution.arc_loads.highlight(&self.fixed_arcs, colored::Color::Blue),
                ))
                .collect::<Vec<String>>()
                .join("\n"),
            solutions.cost(&self.costs, &self.options.cost_fn)
        )
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string_repr: Vec<String> = vec![];
        string_repr.push("\nNetwork:".to_string());
        string_repr.push("========".to_string());
        string_repr.push(format!(
            "Vertices: ({})",
            self.vertices
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        string_repr.push(format!("Capacities:\n{}", self.capacities));
        string_repr.push(format!("Costs:\n{}\n", self.costs));
        string_repr.push(format!("{} Scenarios:", self.balances.len()));
        self.balances.iter().enumerate().for_each(|(i, b)| {
            string_repr.push(format!("{}.:\n{}", i, b));
        });
        string_repr.push("".to_string());
        string_repr.push(format!(
            "The following arcs have been marked as fixed: {}",
            self.fixed_arcs
                .iter()
                .map(|(s, t)| format!("({}->{})", self.vertices[*s], self.vertices[*t]))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        string_repr.push("".to_string());
        if let Some(baseline) = &self.baseline {
            string_repr.push(
                    format!("The lower bound on network cost is {}. Omitting consistent flow constraints yields the following consistent flows:\n{}",
                    baseline.cost(&self.costs, &self.options.cost_fn),
                    baseline.consistent_flows_colorized(&self.fixed_arcs, Color::Blue))
                );
            string_repr.push("".to_string());
        };
        if let (Some(baseline), Some(solutions)) = (&self.baseline, &self.solutions) {
            string_repr.push(
                    format!("Introducing consistency constraints leads to the following consistent flow values:\n{}",
                    solutions.consistent_flows_colorized(&self.fixed_arcs, Color::Blue))
                );
            string_repr.push("".to_string());
            string_repr.push(format!(
                "This corresponds to a relative change in cost of {} and the following relative changes in consistent flows:\n{}",
                (solutions.cost(&self.costs, &self.options.cost_fn) as i64)
                    - (baseline.cost(&self.costs, &self.options.cost_fn) as i64),
                solutions.highlight_difference_to(baseline, &self.fixed_arcs)
            ));
            string_repr.push("".to_string());
        }
        if let (None, Some(solutions)) = (&self.baseline, &self.solutions) {
            string_repr.push(format!(
                "The consistent flow values are:\n{}",
                solutions.consistent_flows_colorized(&self.fixed_arcs, Color::Blue)
            ));
            string_repr.push("".to_string());
        }
        string_repr.push(match &self.solutions {
            Some(_) => format!(
                "{}\n{}",
                &self.display_solutions(),
                &self.options.remainder_solve_method
            ),
            None => "Solution has not been calculated yet.".to_string(),
        });
        write!(f, "{}", string_repr.join("\n"))
    }
}
