mod auxiliary_network;
mod b_tuple;
mod network_state;
mod preprocessing;
mod scenario;
mod solution;
mod vertex;

use colored::Color;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;

use crate::{
    algorithms::{greedy, gurobi},
    matrix::Matrix,
    options::RemainderSolveMethod,
    Options,
};
use crate::{Result, SolverError};
pub(super) use auxiliary_network::AuxiliaryNetwork;
pub(crate) use scenario::Scenario;
pub(super) use solution::{ScenarioSolution, Solution};
pub use vertex::Vertex;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Network {
    pub vertices: Vec<Vertex>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,

    #[serde(skip)]
    pub(crate) baseline: Option<Vec<ScenarioSolution>>,
    #[serde(skip)]
    pub(crate) solutions: Option<Vec<ScenarioSolution>>,
    #[serde(skip)]
    pub(crate) auxiliary_network: Option<AuxiliaryNetwork>,

    #[serde(skip)]
    pub options: Options,
}

impl Network {
    pub fn from_file(options: &Options, filename: &str) -> Result<Self> {
        let network_string = fs::read_to_string(filename)?;
        let mut network: Network = serde_json::from_str(&network_string)?;
        network.options = options.clone();
        Ok(network)
    }

    pub fn serialize(&self, filename: &str) -> Result<()> {
        let json_str = serde_json::to_string(self)?;
        log::debug!("Writing\n{json_str}\nto {filename}");
        std::fs::write(filename, json_str)?;
        Ok(())
    }

    pub fn validate_network(&self) -> Result<()> {
        let len = self.vertices.len();

        let matrices = [&self.capacities, &self.costs];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                return Err(SolverError::NetworkShapeError(
                    "capacities, and costs have differing dimensions or are not quadratic"
                        .to_owned(),
                ));
            }
        }

        for (i, row) in self.capacities.as_rows().iter().enumerate() {
            if row.iter().sum::<usize>() == 0 {
                return Err(SolverError::NetworkShapeError(format!(
                    "vertex {} is a dead end",
                    self.vertices[i]
                )));
            }
        }
        for (i, column) in self.capacities.as_columns().iter().enumerate() {
            if column.iter().sum::<usize>() == 0 {
                return Err(SolverError::NetworkShapeError(format!(
                    "vertex {} is unreachable",
                    self.vertices[i]
                )));
            }
        }

        let total_capacity = self.capacities.sum();
        for (i, matrix) in self.balances.iter().enumerate() {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                return Err(SolverError::NetworkShapeError(
                    "some balances have differing dimensions or are not quadratic".to_owned(),
                ));
            }

            for (s, t) in matrix.indices() {
                if s == t && *matrix.get(s, t) > 0 {
                    return Err(SolverError::NetworkShapeError(format!(
                        "self-supply ({}->{}) is not allowed",
                        self.vertices[s], self.vertices[t]
                    )));
                }
            }

            let as_columns = matrix.as_columns();
            for (j, row) in matrix.as_rows().iter().enumerate() {
                if row.iter().sum::<usize>() != as_columns[j].clone().into_iter().sum::<usize>() {
                    log::warn!(
                        "Vertex {} has supply {}, but demand {} in scenario {}.",
                        self.vertices[j],
                        row.iter().sum::<usize>(),
                        as_columns[j].clone().into_iter().sum::<usize>(),
                        i
                    );
                }
            }

            if total_capacity < matrix.sum() {
                return Err(SolverError::NetworkShapeError(format!(
                    "scenario {i} has higher supply than the network has capacities"
                )));
            }
        }

        log::info!("Network is valid.");
        Ok(())
    }

    pub fn preprocess(&mut self) -> Result<()> {
        log::info!("Beginning to preprocess network...");
        match self.auxiliary_network {
            Some(_) => {}
            None => {
                let auxiliary_network = AuxiliaryNetwork::from_network(&*self)?;
                self.auxiliary_network = Some(auxiliary_network);
            }
        }
        log::info!("Preprocessing complete.");
        Ok(())
    }

    pub fn lower_bound(&mut self) -> Result<()> {
        log::info!("Attempting to find a lower bound for network cost...");
        let capacities_memory = self.capacities.clone();
        for (s, t) in self.fixed_arcs.iter() {
            self.capacities.set(*s, *t, usize::MAX);
        }
        match gurobi(self) {
            Err(e) => {
                self.capacities = capacities_memory;
                Err(e)
            }
            Ok(solutions) => {
                self.baseline = Some(solutions);
                self.capacities = capacities_memory;
                log::info!("Found a lower bound on network cost.");
                Ok(())
            }
        }
    }

    pub fn solve(&mut self) -> Result<()> {
        log::info!("Attempting to find a feasible robust flow...");
        let auxiliary_network = match &mut self.auxiliary_network {
            Some(aux) => aux,
            None => return Err(SolverError::SkippedPreprocessingError),
        };
        log::debug!("Found auxiliary network, calling greedy on it...");

        match greedy(auxiliary_network, &self.options) {
            Ok(solutions) => {
                self.solutions = Some(solutions);
                log::info!("Found a solution.");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn solve_remainder(&mut self) -> Result<()> {
        match self.options.remainder_solve_method {
            RemainderSolveMethod::None => log::info!("Skipping solve of remaining network."),
            RemainderSolveMethod::Greedy => log::debug!("No need to solve remaining network."),
            RemainderSolveMethod::Gurobi => {
                log::info!("Passing the remaining unsolved network to Gurobi...");
                let solutions = gurobi(self)?;
                self.solutions = Some(solutions);
            }
        }
        Ok(())
    }

    pub fn validate_solution(&self) -> Result<()> {
        log::info!("Attempting to assess validity of found solution...");
        let solutions = match &self.solutions {
            Some(solutions) => solutions,
            None => return Err(SolverError::SkippedSolveError),
        };
        log::debug!("Found existing solution, validating...");

        if solutions.len() != self.balances.len() {
            return Err(SolverError::NetworkShapeError(format!(
                "Found {} scenario solutions, but expected {}.",
                solutions.len(),
                self.balances.len()
            )));
        }

        for (i, _) in self.balances.iter().enumerate() {
            for (s, t) in self
                .capacities
                .indices()
                .filter(|(s, t)| s != t && !self.fixed_arcs.contains(&(*s, *t)))
            {
                if self.capacities.get(s, t) < solutions[i].arc_loads.get(s, t) {
                    return Err(SolverError::NetworkShapeError(format!(
                        "Scenario {} puts load {} on arc ({}->{}), but its capacity is {}.",
                        i,
                        solutions[i].arc_loads.get(s, t),
                        self.vertices[s],
                        self.vertices[t],
                        self.capacities.get(s, t)
                    )));
                }
            }
        }

        log::info!("Solution is valid.");
        Ok(())
    }

    fn display_solutions(&self) -> String {
        let Some(solutions) = &self.solutions else {
            return "No solutions have been found yet.".to_string();
        };
        format!(
            "The following arc loads constitute the solution:\n{}\nThe network cost is {}.",
            solutions.iter()
                .map(|solution| format!(
                    "Scenario {}, with cost {} and {}/{} slack used in delivery of {}/{} supply units:\n{}",
                    solution.id,
                    solution.cost(&self.costs),
                    solution.slack_used(),
                    solution.slack_total,
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
