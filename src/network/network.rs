use crate::{algorithms::greedy, matrix::Matrix};
use core::panic;
use serde::Deserialize;
use serde_json;
use std::{fmt::Display, fs::File, io::BufReader};

use super::{auxiliary_network::AuxiliaryNetwork, solution::Solution};

#[derive(Deserialize, Debug)]
pub struct Network {
    pub vertices: Vec<String>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,
    #[serde(skip)]
    pub arc_loads: Solution,
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

    pub fn validate_network(&self) {
        let len = self.vertices.len();

        let matrices = [&self.capacities, &self.costs];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices u, c have differing dimensions or are not quadratic");
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
                        i + 1
                    );
                }
            }

            if total_capacity < matrix.sum() {
                panic!("No feasible soution exists: balance {} has higher supply than the network has capacities.", i+1);
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
        self.arc_loads = match auxiliary_network {
            Some(mut aux) => {
                log::debug!("Found auxiliary network, calling greedy on it...");
                greedy(&mut aux);
                Solution::from(&*aux)
            }
            None => {
                log::error!("No auxiliary network found. Forgot to preprocess?");
                Solution::default()
            }
        }
    }

    pub fn validate_solution(&self) {
        if self.arc_loads.get().is_none() {
            log::error!("Solution is empty. Forgot to solve?");
            return;
        }
        log::info!("Assessing validity of found solution...");

        let arc_loads = self.arc_loads.get().unwrap();
        if arc_loads.len() != self.balances.len() {
            log::error!(
                "Found {} scenario solutions, but expected {}.",
                arc_loads.len(),
                self.balances.len()
            );
        }

        for (i, _) in self.balances.iter().enumerate() {
            for (s, t) in self
                .capacities
                .indices()
                .filter(|(s, t)| s != t && !self.fixed_arcs.contains(&(*s, *t)))
            {
                if self.capacities.get(s, t) < arc_loads[i].get(s, t) {
                    log::error!("Scenario {} places an arc load of {} on arc ({}->{}), but this arc only has capacity {}.", i+1, arc_loads[i].get(s, t), self.vertices[s], self.vertices[t], self.capacities.get(s, t));
                }
            }
        }
        log::info!("Validity check complete.");
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string_repr: Vec<String> = vec![];
        string_repr.push("Network:".to_string());
        string_repr.push("========".to_string());
        string_repr.push(format!("Vertices: ({})", self.vertices.join(", ")));
        string_repr.push(format!("Capacities:\n{:>8}", self.capacities));
        string_repr.push(format!("Costs:\n{:>8}", self.costs));
        string_repr.push(format!("{} Scenarios:", self.balances.len()));
        self.balances.iter().enumerate().for_each(|(i, b)| {
            string_repr.push(format!("{}.:\n{:>8}", i + 1, b));
        });
        string_repr.push(format!("The following arcs have been marked as fixed:"));
        string_repr.push(format!(
            "{:>8}",
            self.fixed_arcs
                .iter()
                .map(|(s, t)| format!("({}->{})", self.vertices[*s], self.vertices[*t]))
                .collect::<Vec<String>>()
                .join(", ")
        ));
        string_repr.push(format!("{}", self.arc_loads));
        write!(f, "{}", string_repr.join("\n"))
    }
}
