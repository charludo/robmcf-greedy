use std::fmt::Display;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum RemainderSolveMethod {
    None,
    Greedy,
    Gurobi,
    #[clap(skip)]
    Ilp,
}

impl Display for RemainderSolveMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Only relevant (s, t) pairs were considered in the greedy solution. The remainder of the network has not been solved."),
            Self::Greedy => write!(f, "The entire network was solved greedily."),
            Self::Gurobi => write!(f, "Only relevant (s, t) pairs were considered in the greedy solution. The remainder of the network has been solved via an ILP."),
            Self::Ilp => write!(f, "The entire network has been solved via ILP.")
        }
    }
}
