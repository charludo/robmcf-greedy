mod display;
mod export;
mod random;
mod solution;
mod to_latex;
mod vertex;

use serde::{Deserialize, Serialize};
use std::fs;

pub(super) use crate::auxiliary::AuxiliaryNetwork;
use crate::{options::RemainderSolveMethod, Matrix, Options};
use crate::{Result, SolverError};
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

    pub fn export(
        &self,
        filename: &str,
        time_preprocess: Option<usize>,
        time_solve: Option<usize>,
    ) -> Result<()> {
        match &self.solutions {
            Some(_) => {}
            None => return Err(SolverError::SkippedSolveError),
        };
        let file_exists = std::path::Path::new(filename).exists();
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)?;
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);
        let data = export::NetworkData::from_network(self, time_preprocess, time_solve);
        wtr.serialize(&data)?;
        wtr.flush()?;
        log::debug!("Writing\n{:?}\ntp {filename}", data);
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
                        "({}): Vertex {} has supply {}, but demand {}.",
                        i,
                        self.vertices[j],
                        row.iter().sum::<usize>(),
                        as_columns[j].clone().into_iter().sum::<usize>(),
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
        match crate::ilp::gurobi_partial(self, true) {
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

    pub fn original_flow(&mut self) -> Result<()> {
        log::info!("Attempting to calculate the original flow...");
        match crate::ilp::gurobi_partial(self, false) {
            Ok(solutions) => {
                self.baseline = Some(solutions);
                log::info!("Calculated the original flow.");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn fix_best_candidates(&mut self, n: usize) -> Result<()> {
        log::info!("Attempting to find and fix promising improvement candidates...");
        let baseline = match &self.baseline {
            Some(baseline) => baseline,
            None => return Err(SolverError::SkippedBaselineError),
        };

        let mut candidates = Matrix::filled_with(0, self.vertices.len(), self.vertices.len());
        for scenario in baseline {
            for (i, j) in scenario.arc_loads.indices() {
                let arc_load = *scenario.arc_loads.get(i, j);
                let capacity = *self.capacities.get(i, j);
                if capacity > 0 && arc_load == capacity {
                    candidates.increment(i, j);
                }
            }
        }

        let mut heap = std::collections::BinaryHeap::with_capacity(n);
        for (i, j) in candidates.indices() {
            let value = candidates.get(i, j);
            if heap.len() < n {
                heap.push(std::cmp::Reverse((value, (i, j))));
            } else if let Some(&std::cmp::Reverse((min_value, _))) = heap.peek() {
                if value > min_value {
                    heap.pop();
                    heap.push(std::cmp::Reverse((value, (i, j))));
                }
            }
        }
        let candidates = heap
            .into_iter()
            .map(|std::cmp::Reverse((_, index))| index)
            .collect::<Vec<_>>();
        let candidate_count = candidates
            .iter()
            .filter(|c| *self.capacities.get(c.0, c.1) > 0)
            .count();
        if candidate_count < n {
            return Err(SolverError::NoCandidatesError(candidate_count, n));
        }

        log::debug!(
            "Found the following improvement candidates: {:?}",
            candidates
        );
        self.fixed_arcs = candidates;

        log::info!("Fixed the {n} most promising improvement candidate arcs.");
        Ok(())
    }

    pub fn add_penalty_arcs(&mut self) -> Result<()> {
        let indices = self
            .capacities
            .indices()
            .filter(|&(s, t)| *self.capacities.get(s, t) == 0)
            .collect::<Vec<_>>();
        for (s, t) in indices {
            if self.balances.iter().any(|b| *b.get(s, t) > 0) {
                self.capacities.set(s, t, usize::MAX);
                self.costs.set(s, t, usize::MAX / 2);
            }
        }
        log::info!("Added penalty arcs.");
        Ok(())
    }

    pub fn solve(&mut self) -> Result<()> {
        log::info!("Attempting to find a feasible robust flow...");
        let auxiliary_network = match &mut self.auxiliary_network {
            Some(aux) => aux,
            None => return Err(SolverError::SkippedPreprocessingError),
        };
        log::debug!("Found auxiliary network, calling greedy on it...");

        match crate::algorithms::greedy(auxiliary_network, &self.options) {
            Ok(solutions) => {
                self.solutions = Some(solutions);
                log::info!("Found a solution.");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn solve_full_ilp(&mut self) -> Result<()> {
        log::info!("Attempting to solve the network as an ILP...");
        match crate::ilp::gurobi_full(self) {
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
            RemainderSolveMethod::Ilp => log::error!("How did you get here??"),
            RemainderSolveMethod::Greedy => log::debug!("No need to solve remaining network."),
            RemainderSolveMethod::Gurobi => {
                log::info!("Passing the remaining unsolved network to Gurobi...");
                let solutions = crate::ilp::gurobi_partial(self, false)?;
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

        for solution in solutions {
            for (s, t) in self
                .capacities
                .indices()
                .filter(|&(s, t)| s != t && !self.fixed_arcs.contains(&(s, t)))
            {
                if *self.capacities.get(s, t) < *solution.arc_loads.get(s, t) {
                    return Err(SolverError::NetworkShapeError(format!(
                        "Scenario {} puts load {} on arc ({}->{}), but its capacity is {}.",
                        solution.id,
                        solution.arc_loads.get(s, t),
                        self.vertices[s],
                        self.vertices[t],
                        self.capacities.get(s, t)
                    )));
                }
            }
        }

        for solution in solutions {
            let total_supply = self.balances[solution.id].sum();
            let excessions = solution
                .arc_loads
                .indices()
                .filter(|(s, t)| *solution.arc_loads.get(*s, *t) > total_supply)
                .collect::<Vec<_>>();
            if !excessions.is_empty() {
                return Err(SolverError::NetworkShapeError(format!(
                    "Scenario {} has {} supply, but the following arcs exceed this: {:?}",
                    solution.id, total_supply, excessions
                )));
            }
        }

        log::info!("Solution is valid.");
        Ok(())
    }
}
