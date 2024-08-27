use crate::{algorithms::greedy, matrix::Matrix};
use core::panic;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fmt::Display, fs::File, io::BufReader};

use super::{auxiliary_network::AuxiliaryNetwork, solution::Solution, vertex::Vertex};

#[derive(Deserialize, Debug, Serialize)]
pub struct Network {
    pub vertices: Vec<Vertex>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,
    #[serde(skip)]
    pub(crate) solution: Option<Solution>,
    #[serde(skip)]
    pub(crate) auxiliary_network: Option<Box<AuxiliaryNetwork>>,
}

impl Network {
    pub fn from_file(filename: &str) -> Self {
        let file = match File::open(filename) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to open file \"{}\": {}", filename, msg),
        };
        let reader = BufReader::new(file);

        log::debug!("Deserializing network from {}", filename);
        match serde_json::from_reader(reader) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to parse the network: {}", msg),
        }
    }

    pub fn serialize(&self, filename: &str) {
        let json_str = serde_json::to_string(self).unwrap();
        log::debug!("Writing\n{json_str}\nto {filename}");
        std::fs::write(filename.to_string(), json_str).unwrap();
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
                if row.into_iter().sum::<usize>()
                    != as_columns[j].clone().into_iter().sum::<usize>()
                {
                    log::warn!(
                        "Vertex {} has supply {}, but demand {} in scenario {}.",
                        self.vertices[j],
                        row.into_iter().sum::<usize>(),
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
            None => self.auxiliary_network = Some(Box::new(AuxiliaryNetwork::from(&*self))),
        }
    }

    pub fn solve(&mut self) {
        log::info!("Attempting to find a feasible robust flow...");
        let auxiliary_network = std::mem::take(&mut self.auxiliary_network);
        self.solution = match auxiliary_network {
            Some(mut aux) => {
                log::debug!("Found auxiliary network, calling greedy on it...");
                greedy(&mut aux);
                self.auxiliary_network = Some(aux);
                Some(Solution::from(&*self))
            }
            None => {
                log::error!("No auxiliary network found. Forgot to preprocess?");
                self.auxiliary_network = auxiliary_network;
                None
            }
        }
    }

    pub fn validate_solution(&self) {
        match &self.solution {
            None => log::error!("Solution is empty. Forgot to solve?"),
            Some(solution) => {
                log::info!("Assessing validity of found solution...");

                if solution.arc_loads.len() != self.balances.len() {
                    log::error!(
                        "Found {} scenario solutions, but expected {}.",
                        solution.arc_loads.len(),
                        self.balances.len()
                    );
                }

                for (i, _) in self.balances.iter().enumerate() {
                    for (s, t) in self
                        .capacities
                        .indices()
                        .filter(|(s, t)| s != t && !self.fixed_arcs.contains(&(*s, *t)))
                    {
                        if self.capacities.get(s, t) < solution.arc_loads[i].get(s, t) {
                            log::error!(
                                "Scenario {} puts load {} on arc ({}->{}), but its capacity is {}.",
                                i,
                                solution.arc_loads[i].get(s, t),
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
        string_repr.push(format!("The following arcs have been marked as fixed:"));
        string_repr.push(format!(
            "{}",
            self.fixed_arcs
                .iter()
                .map(|(s, t)| format!("({}->{})", self.vertices[*s], self.vertices[*t]))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        string_repr.push(match &self.solution {
            Some(solution) => format!("{}", solution),
            None => "Solution has not been calculated yet.".to_string(),
        });
        write!(f, "{}", string_repr.join("\n"))
    }
}
