use std::fmt::Display;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum RemainderSolveMethod {
    No,
    Greedy,
    IntegerProgram,
}

impl Display for RemainderSolveMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::No => write!(f, "Only relevant (s, t) pairs were considered in the greedy solution. The remainder of the network has not been solved."),
            Self::Greedy => write!(f, "The entire network was solved greedily."),
            Self::IntegerProgram => write!(f, "Only relevant (s, t) pairs were considered in the greedy solution. The remainder of the network has been solved via an ILP."),
        }
    }
}
