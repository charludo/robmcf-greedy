use std::{error::Error, fmt::Display};

pub type Result<T> = std::result::Result<T, SolverError>;

#[derive(Debug)]
pub enum SolverError {
    NetworkIOError(std::io::Error),
    NetworkSerializationError(serde_json::Error),
    NetworkShapeError(String),

    FixedArcMemoryCorruptError,
    PathMatrixCorruptError,

    NoFeasibleFlowError(usize),
    NoSlackLeftError(usize),
    GurobiOpsError(grb::Error),
    GurobiSolutionError(usize),

    SkippedPreprocessingError,
    SkippedSolveError,
    InvalidSolutionError(String),
}

impl Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SolverError::NetworkIOError(e) => format!("Failed to read network from file: {e}."),
                SolverError::NetworkSerializationError(e) =>
                    format!("Failed to parse the network: {e}."),
                SolverError::NetworkShapeError(e) => format!("Network is invalid: {e}"),
                SolverError::FixedArcMemoryCorruptError =>
                    "The fixed arc memory is corrupted.".to_owned(),
                SolverError::PathMatrixCorruptError =>
                    "The shortest path matrix is corrupted.".to_owned(),
                SolverError::NoFeasibleFlowError(e) =>
                    format!("No feasible flow could be found in scenario {e}."),
                SolverError::NoSlackLeftError(e) =>
                    format!("Slack allowance exceeded in scenario {e}."),
                SolverError::GurobiOpsError(e) => format!("Gurobi encountered an error: {e}."),
                SolverError::GurobiSolutionError(e) =>
                    format!("Gurobi could not find a feasible flow in scenario {e}."),
                SolverError::SkippedPreprocessingError =>
                    "No auxiliary network found. Forgot to preprocess?".to_owned(),
                SolverError::SkippedSolveError =>
                    "No solution found for validation. Forgot to solve?".to_owned(),
                SolverError::InvalidSolutionError(e) =>
                    format!("Found a solution, but it is invalid: {e}."),
            }
        )
    }
}

impl Error for SolverError {}

impl From<serde_json::Error> for SolverError {
    fn from(value: serde_json::Error) -> Self {
        SolverError::NetworkSerializationError(value)
    }
}

impl From<std::io::Error> for SolverError {
    fn from(value: std::io::Error) -> Self {
        SolverError::NetworkIOError(value)
    }
}

impl From<grb::Error> for SolverError {
    fn from(value: grb::Error) -> Self {
        SolverError::GurobiOpsError(value)
    }
}
