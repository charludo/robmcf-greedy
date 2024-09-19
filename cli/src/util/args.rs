use clap::{Parser, Subcommand};
use robmcf_greedy::{
    CostFunction, DeltaFunction, RelativeDrawFunction, RemainderSolveMethod, SlackFunction,
};

/// CLI for the Greedy RobMCF solver library.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,

    /// Enable [v]erbose debug logging
    #[arg(long, short = 'v', global = true, display_order = 1)]
    pub(crate) debug: bool,

    /// Enable trace logging
    #[arg(long, short, global = true, display_order = 2)]
    pub(crate) trace: bool,

    /// Disable logging. Takes precedence over debug.
    #[arg(long, short, long, global = true, display_order = 3)]
    pub(crate) quiet: bool,

    /// Function used to calculate the [c]ost of the overall solution
    #[arg(long, short='c', value_enum, default_value_t = CostFunction::Max, global = true, display_order = 11, help_heading="Solver Parameters")]
    pub(crate) costs: CostFunction,

    /// [D]istance function used in determining intermediate arc sets
    #[arg(long, short='d', value_enum, default_value_t = DeltaFunction::LogarithmicMedium, global = true, display_order = 12, help_heading="Solver Parameters")]
    pub(crate) delta: DeltaFunction,

    /// Function used to calculate the [r]elative draw of supply towards fixerd arcs
    #[arg(long, short='r', value_enum, default_value_t = RelativeDrawFunction::PeerPressure, global = true, display_order = 13, help_heading="Solver Parameters")]
    pub(crate) draw: RelativeDrawFunction,

    /// Function used in determining the total [s]lack available to scenarios
    #[arg(long, short='s', value_enum, default_value_t = SlackFunction::DifferenceToMaxPlus_10, global = true, display_order = 14, help_heading="Solver Parameters")]
    pub(crate) slack: SlackFunction,

    /// [M]ethod by which a solution for routing supply which cannot use fixed arcs is found
    #[arg(long, short='m', value_enum, default_value_t = RemainderSolveMethod::None, global = true, display_order = 15, help_heading="Solver Parameters")]
    pub(crate) remainder: RemainderSolveMethod,
}

#[derive(Parser, Debug)]
pub(crate) struct RandomizationArgs {
    /// The fraction of arcs with a capacity greater than zero
    #[arg(
        long,
        default_value_t = 0.4,
        display_order = 101,
        help_heading = "Random Capacities"
    )]
    pub(crate) arc_density: f64,

    /// Minimum capacity of generated arcs
    #[arg(
        long,
        default_value_t = 10,
        display_order = 102,
        help_heading = "Random Capacities"
    )]
    pub(crate) umin: usize,

    /// Maximum capacity of generated arcs
    #[arg(
        long,
        default_value_t = 30,
        display_order = 103,
        help_heading = "Random Capacities"
    )]
    pub(crate) umax: usize,

    /// Minimum arc cost
    #[arg(
        long,
        default_value_t = 4,
        display_order = 201,
        help_heading = "Random Costs"
    )]
    pub(crate) cmin: usize,

    /// Maximum arc cost
    #[arg(
        long,
        default_value_t = 8,
        display_order = 202,
        help_heading = "Random Costs"
    )]
    pub(crate) cmax: usize,

    /// Number of scenarios to generate
    #[arg(
        long,
        default_value_t = 2,
        display_order = 301,
        help_heading = "Random Scenarios"
    )]
    pub(crate) scenarios: usize,

    /// Fraction of vertices each vertex has supply greater than zero for
    #[arg(
        long,
        default_value_t = 0.3,
        display_order = 302,
        help_heading = "Random Scenarios"
    )]
    pub(crate) supply_density: f64,

    /// Minimum supply value
    #[arg(
        long,
        default_value_t = 2,
        display_order = 303,
        help_heading = "Random Scenarios"
    )]
    pub(crate) bmin: usize,

    /// Maximum supply value
    #[arg(
        long,
        default_value_t = 8,
        display_order = 304,
        help_heading = "Random Scenarios"
    )]
    pub(crate) bmax: usize,

    /// Number of fixed arcs
    #[arg(
        long,
        default_value_t = 5,
        display_order = 401,
        help_heading = "Random Fixed Arcs"
    )]
    pub(crate) fixed: usize,

    /// Force fixed arcs to be consecutive, i.e. share one vertex with the next fixed arc
    #[arg(long, display_order = 401, help_heading = "Random Fixed Arcs")]
    pub(crate) fixed_consecutive: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Solve RobMCF greedily for the given network.
    Solve {
        /// Path to a file containing a network to be used as input.
        file: String,

        /// Path to output file to save the network in
        #[arg(short, long, display_order = 0)]
        output: Option<String>,

        /// Skip calculation of [l]ower bound for network costs. Required if Gurobi is not available.
        #[arg(long, short = 'l', global = true, display_order = 1)]
        skip_lower_bound: bool,

        /// Enable capacity randomization
        #[arg(long, display_order = 100, help_heading = "Random Capacities")]
        randomize_capacities: bool,

        /// Enable cost randomization
        #[arg(long, display_order = 200, help_heading = "Random Costs")]
        randomize_costs: bool,

        /// Enable scenario randomization
        #[arg(long, display_order = 300, help_heading = "Random Scenarios")]
        randomize_scenarios: bool,

        /// Enable fixed arc randomization
        #[arg(long, display_order = 400, help_heading = "Random Fixed Arcs")]
        randomize_fixed_arcs: bool,

        #[command(flatten)]
        random: RandomizationArgs,
    },
    /// Attempt to solve the entire network via an ILP. No greedy involvement.
    Ilp {
        /// Path to a file containing a network to be used as input.
        file: String,
    },
    /// Benchmark the solution process. Should use "No" or "Greedy" for the remainder function.
    Benchmark {
        /// Path to a file containing a network to be used as input.
        file: String,

        /// Number of iterations over which to average
        #[arg(short, long, display_order = 0)]
        iterations: usize,
    },
    /// Create a completely random network instead of using an input file.
    Random {
        /// Path to output file to save the network in
        #[arg(short, long, display_order = 0)]
        output: Option<String>,

        /// Skip calculation of [l]ower bound for network costs. Required if Gurobi is not available.
        #[arg(long, short = 'l', global = true, display_order = 1)]
        skip_lower_bound: bool,

        /// Number of vertices
        vertices: usize,

        #[command(flatten)]
        random: RandomizationArgs,
    },
}
