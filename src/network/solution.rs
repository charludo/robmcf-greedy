use std::fmt::Display;

use crate::matrix::Matrix;

use super::AuxiliaryNetwork;

#[derive(Default, Debug)]
pub(crate) struct Solution(Option<Vec<Matrix<usize>>>);

impl Solution {
    pub(crate) fn get(&self) -> Option<Vec<Matrix<usize>>> {
        self.0.clone()
    }
}

impl From<&AuxiliaryNetwork> for Solution {
    fn from(network: &AuxiliaryNetwork) -> Self {
        let mut solution: Vec<Matrix<usize>> = Vec::new();
        network.scenarios.iter().for_each(|scenario| {
            let mut arc_loads = scenario.arc_loads.clone();
            network.fixed_arcs.iter().for_each(|fixed_arc| {
                let original_arc_start = network.fixed_arcs_memory.get(&fixed_arc.0).unwrap();
                arc_loads.set(
                    *original_arc_start,
                    fixed_arc.1,
                    *arc_loads.get(fixed_arc.0, fixed_arc.1),
                );
            });
            arc_loads.shrink(network.fixed_arcs.len());
            solution.push(arc_loads);
        });
        Solution(Some(solution))
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "Solution has not been calculated yet."),
            Some(loads) => write!(
                f,
                "The following arc loads constitute the solution:\n{}",
                loads
                    .iter()
                    .enumerate()
                    .map(|(i, load)| format!("Scenario {}:\n{}", i + 1, load))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        }
    }
}
