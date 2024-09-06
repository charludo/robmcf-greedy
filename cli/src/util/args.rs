use clap::{Parser, Subcommand};
use robmcf_greedy::options::*;

/// CLI for the Greedy RobMCF solver library.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    pub(crate) debug: bool,

    /// Disable logging. Takes precendence over debug.
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,

    /// Function used to calculate the cost of the overall solution
    #[arg(long, value_enum, default_value_t = CostFunction::Max, global = true)]
    pub(crate) costs: CostFunction,

    /// Function used in determining intermediate arc sets
    #[arg(long, value_enum, default_value_t = DeltaFunction::LinearMedium, global = true)]
    pub(crate) delta: DeltaFunction,

    /// Function used to calculate the relative draw of supply towards fixerd arcs
    #[arg(long, value_enum, default_value_t = RelativeDrawFunction::Linear, global = true)]
    pub(crate) draw: RelativeDrawFunction,

    /// Function used in determining the total slack available to scenarios
    #[arg(long, value_enum, default_value_t = SlackFunction::DifferenceToMax, global = true)]
    pub(crate) slack: SlackFunction,

    /// Method by which a solution for routing supply which cannot use fixed arcs is found
    #[arg(long, value_enum, default_value_t = RemainderSolveMethod::No, global = true)]
    pub(crate) remainder: RemainderSolveMethod,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Solve RobMCF greedily for the given network.
    Solve {
        /// Path to a file containing a network to be used as input.
        file: String,
    },
    /// Benchmark the solution process. Should use "No" or "Greedy" for the remainder function.
    Benchmark {
        /// Path to a file containing a network to be used as input.
        file: String,

        /// Number of iterations over which to average
        #[arg(short, long)]
        iterations: usize,
    },
    /// Create a random network instead of using an input file.
    Random {
        /// Path to output file to save the generated network in
        #[arg(short, long)]
        output: Option<String>,

        /// Number of vertices
        #[arg(short, long, default_value_t = 10)]
        vertices: usize,

        /// Number of fixed arcs
        #[arg(short, long, default_value_t = 5)]
        fixed: usize,

        /// Connectedness as the fraction of arcs with a capacity greater than zero
        #[arg(short, long, default_value_t = 0.4)]
        arc_density: f64,

        /// Minimum capacity of generated arcs
        #[arg(long, default_value_t = 10)]
        umin: usize,

        /// Maximum capacity of generated arcs
        #[arg(long, default_value_t = 30)]
        umax: usize,

        /// Minium arc cost
        #[arg(long, default_value_t = 4)]
        cmin: usize,

        /// Maximum arc cost
        #[arg(long, default_value_t = 8)]
        cmax: usize,

        /// Number of scenarios to generate
        #[arg(short, long, default_value_t = 2)]
        scenarios: usize,

        /// Fraction of vertices each vertex has supply greater than zero for
        #[arg(short, long, default_value_t = 0.3)]
        balance_density: f64,

        /// Minimum supply value
        #[arg(long, default_value_t = 2)]
        bmin: usize,

        /// Maximum supply value
        #[arg(long, default_value_t = 8)]
        bmax: usize,
    },
}
