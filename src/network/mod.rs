mod auxiliary_network;
mod b_tuple;
mod baseline;
mod network_state;
mod preprocessing;
mod scenario;
mod solution;
mod vertex;

use baseline::Baseline;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File, io::BufReader};

use crate::{
    algorithms::{greedy, gurobi},
    matrix::Matrix,
    options::RemainderSolveMethod,
    Options,
};
pub(super) use auxiliary_network::AuxiliaryNetwork;
pub(super) use solution::ScenarioSolution;
pub use vertex::Vertex;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Network {
    pub vertices: Vec<Vertex>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,

    #[serde(skip)]
    pub(crate) baseline: Option<Baseline>,
    #[serde(skip)]
    pub(crate) solutions: Option<Vec<ScenarioSolution>>,
    #[serde(skip)]
    pub(crate) auxiliary_network: Option<AuxiliaryNetwork>,

    #[serde(skip)]
    pub options: Options,
}

impl Network {
    pub fn from_file(options: &Options, filename: &str) -> Self {
        let file = match File::open(filename) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to open file \"{}\": {}", filename, msg),
        };
        let reader = BufReader::new(file);

        log::debug!("Deserializing network from {}", filename);
        let mut network: Network = match serde_json::from_reader(reader) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to parse the network: {}", msg),
        };

        network.options = options.clone();
        network
    }

    pub fn serialize(&self, filename: &str) {
        let json_str = serde_json::to_string(self).unwrap();
        log::debug!("Writing\n{json_str}\nto {filename}");
        std::fs::write(filename, json_str).unwrap();
    }

    pub fn validate_network(&self) {
        let len = self.vertices.len();

        let matrices = [&self.capacities, &self.costs];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices u, c have differing dimensions or are not quadratic");
            }
        }

        for (i, row) in self.capacities.as_rows().iter().enumerate() {
            if row.iter().sum::<usize>() == 0 {
                panic!("Vertex {} is a dead end", self.vertices[i]);
            }
        }
        for (i, column) in self.capacities.as_columns().iter().enumerate() {
            if column.iter().sum::<usize>() == 0 {
                panic!("Vertex {} is unreachable", self.vertices[i]);
            }
        }

        let total_capacity = self.capacities.sum();
        for (i, matrix) in self.balances.iter().enumerate() {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices in b have differing dimensions or are not quadratic");
            }

            for (s, t) in matrix.indices() {
                if s == t && *matrix.get(s, t) > 0 {
                    panic!(
                        "Circular supply ({}->{}) is not allowed.",
                        self.vertices[s], self.vertices[t]
                    );
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
                panic!("Balance {i} has higher supply than the network has capacities!");
            }
        }

        log::info!("Network is valid.");
    }

    pub fn preprocess(&mut self) {
        log::info!("Beginning to preprocess network.");
        match self.auxiliary_network {
            Some(_) => {}
            None => self.auxiliary_network = Some(AuxiliaryNetwork::from(&*self)),
        }
    }

    pub fn lower_bound(&mut self) -> Result<(), ()> {
        log::info!("Attempting to find a lower bound for network cost...");
        let capacities_memory = self.capacities.clone();
        for (s, t) in self.fixed_arcs.iter() {
            self.capacities.set(*s, *t, usize::MAX);
        }
        let return_value = match gurobi(self) {
            Err(_) => Err(()),
            Ok(solutions) => {
                self.baseline = Some(Baseline::from_solutions(
                    &solutions,
                    &self.fixed_arcs,
                    &self.costs,
                    &self.options.cost_fn,
                ));
                Ok(())
            }
        };
        self.capacities = capacities_memory;
        return_value
    }

    pub fn solve(&mut self) -> Result<(), ()> {
        log::info!("Attempting to find a feasible robust flow...");
        let Some(auxiliary_network) = &mut self.auxiliary_network else {
            log::error!("No auxiliary network found. Forgot to preprocess?");
            return Err(());
        };
        log::debug!("Found auxiliary network, calling greedy on it...");

        match greedy(auxiliary_network, &self.options) {
            Ok(solutions) => {
                self.solutions = Some(solutions);
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn solve_remainder(&mut self) {
        match self.options.remainder_solve_method {
            RemainderSolveMethod::None => log::info!("Skipping solve of remaining network."),
            RemainderSolveMethod::Greedy => log::debug!("No need to solve remaining network."),
            RemainderSolveMethod::Gurobi => {
                log::info!("Passing the remaining unsolved network to Gurobi...");
                match gurobi(self) {
                    Err(_) => panic!("Gurobi could not find a solution."),
                    Ok(solutions) => self.solutions = Some(solutions),
                };
            }
        }
    }

    pub fn validate_solution(&self) {
        match &self.solutions {
            None => log::error!("Solution is empty. Forgot to solve?"),
            Some(solutions) => {
                log::info!("Assessing validity of found solution...");

                if solutions.len() != self.balances.len() {
                    log::error!(
                        "Found {} scenario solutions, but expected {}.",
                        solutions.len(),
                        self.balances.len()
                    );
                }

                for (i, _) in self.balances.iter().enumerate() {
                    for (s, t) in self
                        .capacities
                        .indices()
                        .filter(|(s, t)| s != t && !self.fixed_arcs.contains(&(*s, *t)))
                    {
                        if self.capacities.get(s, t) < solutions[i].arc_loads.get(s, t) {
                            log::error!(
                                "Scenario {} puts load {} on arc ({}->{}), but its capacity is {}.",
                                i,
                                solutions[i].arc_loads.get(s, t),
                                self.vertices[s],
                                self.vertices[t],
                                self.capacities.get(s, t)
                            );
                        }
                    }
                }
                log::info!("Validity check complete.");
            }
        }
    }

    pub fn cost(&self) -> usize {
        match &self.solutions {
            Some(solutions) => self.options.cost_fn.apply(
                solutions
                    .iter()
                    .map(|s| s.cost(&self.costs))
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            None => 0,
        }
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
            self.cost()
        )
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string_repr: Vec<String> = vec![];
        string_repr.push("Network:".to_string());
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
        string_repr.push(format!("Costs:\n{}", self.costs));
        string_repr.push(format!("{} Scenarios:", self.balances.len()));
        self.balances.iter().enumerate().for_each(|(i, b)| {
            string_repr.push(format!("{}.:\n{}", i, b));
        });
        string_repr.push("The following arcs have been marked as fixed:".to_string());
        string_repr.push(
            self.fixed_arcs
                .iter()
                .map(|(s, t)| format!("({}->{})", self.vertices[*s], self.vertices[*t]))
                .collect::<Vec<String>>()
                .join(", ")
                .to_string(),
        );
        string_repr.push(match &self.baseline {
            Some(baseline) => format!("{}", baseline),
            None => "Lower bound on cost has not been calculated.".to_string(),
        });
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
