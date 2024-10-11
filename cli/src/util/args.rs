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

    /// Enable [t]race logging
    #[arg(long, short, global = true, display_order = 2)]
    pub(crate) trace: bool,

    /// Disable logging, [q]uieting output. Takes precedence over debug.
    #[arg(long, short, long, global = true, display_order = 3)]
    pub(crate) quiet: bool,

    /// [E]xport all resulting measures, appending them to the given file.
    #[arg(long, short, long, global = true, display_order = 4)]
    pub(crate) export: Option<String>,

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
    #[arg(long, short='s', value_enum, default_value_t = SlackFunction::DifferenceToMax, global = true, display_order = 14, help_heading="Solver Parameters")]
    pub(crate) slack: SlackFunction,

    /// [M]ethod by which a solution for routing supply which cannot use fixed arcs is found
    #[arg(long, short='m', value_enum, default_value_t = RemainderSolveMethod::None, global = true, display_order = 15, help_heading="Solver Parameters")]
    pub(crate) remainder: RemainderSolveMethod,
}

#[derive(Parser, Debug)]
pub(crate) struct RandomizationArgs {
    /// The fraction of vertices that are "stations" and can have demand/supply
    #[arg(
        long,
        default_value_t = 1.0,
        display_order = 100,
        help_heading = "Random Vertices"
    )]
    pub(crate) station_density: f64,

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
        default_value_t = 15,
        display_order = 102,
        help_heading = "Random Capacities"
    )]
    pub(crate) umin: usize,

    /// Maximum capacity of generated arcs
    #[arg(
        long,
        default_value_t = 40,
        display_order = 103,
        help_heading = "Random Capacities"
    )]
    pub(crate) umax: usize,

    /// Minimum arc cost
    #[arg(
        long,
        default_value_t = 1,
        display_order = 201,
        help_heading = "Random Costs"
    )]
    pub(crate) cmin: usize,

    /// Maximum arc cost
    #[arg(
        long,
        default_value_t = 12,
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

    /// Minimum fraction of vertices each vertex has supply greater than zero for
    #[arg(
        long,
        default_value_t = 0.01,
        display_order = 302,
        help_heading = "Random Scenarios"
    )]
    pub(crate) supply_density_min: f64,

    /// Maximum fraction of vertices each vertex has supply greater than zero for
    #[arg(
        long,
        default_value_t = 0.2,
        display_order = 303,
        help_heading = "Random Scenarios"
    )]
    pub(crate) supply_density_max: f64,

    /// Minimum supply value
    #[arg(
        long,
        default_value_t = 1,
        display_order = 304,
        help_heading = "Random Scenarios"
    )]
    pub(crate) bmin: usize,

    /// Maximum supply value
    #[arg(
        long,
        default_value_t = 5,
        display_order = 305,
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
    #[arg(long, display_order = 402, help_heading = "Random Fixed Arcs")]
    pub(crate) fixed_consecutive: bool,

    /// Only allow improvement of existing arcs, not the creation of new ones
    #[arg(long, display_order = 403, help_heading = "Random Fixed Arcs")]
    pub(crate) existing_only: bool,

    /// Fix the n [b]est improvement candidates based on the original flow.
    #[arg(
        long,
        short = 'b',
        display_order = 405,
        help_heading = "Random Fixed Arcs",
        conflicts_with_all = ["fixed", "lower_bound"]
    )]
    pub(crate) fix_best: Option<usize>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Solve RobMCF greedily for the given network.
    Solve {
        /// Path to a file containing a network to be used as input.
        file: String,

        /// Path to [o]utput file to save the network in
        #[arg(short, long, display_order = 0)]
        output: Option<String>,

        /// Calculate a [l]ower bound for network costs. Requires Gurobi.
        #[arg(long, short = 'l', global = true, display_order = 1)]
        lower_bound: bool,

        /// Calculate the original [f]low of the network. Requires Gurobi.
        #[arg(
            long,
            short = 'f',
            global = true,
            display_order = 1,
            conflicts_with = "lower_bound"
        )]
        original_flow: bool,

        /// Add [p]enalty arcs between vertices with non-zero supply/demand relationship
        #[arg(long, short = 'p', global = true, display_order = 1)]
        penalty_arcs: bool,

        /// Enable capacity randomization
        #[arg(long, display_order = 100, help_heading = "Random Capacities")]
        randomize_capacities: bool,

        /// Enable cost randomization
        #[arg(long, display_order = 200, help_heading = "Random Costs")]
        randomize_costs: bool,

        /// Override costs. Pass triples of "s,t,cost"
        #[arg(long, display_order = 203, help_heading = "Random Costs", value_parser = parse_triplet, num_args=1..)]
        override_costs: Option<Vec<(usize, usize, usize)>>,

        /// Enable scenario randomization
        #[arg(long, display_order = 300, help_heading = "Random Scenarios")]
        randomize_scenarios: bool,

        /// Enable fixed arc randomization
        #[arg(long, display_order = 400, help_heading = "Random Fixed Arcs")]
        randomize_fixed_arcs: bool,

        #[command(flatten)]
        random: RandomizationArgs,

        /// Override fixed arcs. Pass tuples of "s,t"
        #[arg(long, display_order = 404, help_heading = "Random Fixed Arcs", value_parser = parse_tuple, num_args=1..)]
        override_fixed: Option<Vec<(usize, usize)>>,
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

        /// Number of [i]terations over which to average
        #[arg(short, long, display_order = 0)]
        iterations: usize,
    },
    /// Create a completely random network instead of using an input file.
    Random {
        /// Path to [o]utput file to save the network in
        #[arg(short, long, display_order = 0)]
        output: Option<String>,

        /// Calculate a [l]ower bound for network costs. Requires Gurobi.
        #[arg(long, short = 'l', global = true, display_order = 1)]
        lower_bound: bool,

        /// Calculate the original [f]low of the network. Requires Gurobi.
        #[arg(
            long,
            short = 'f',
            global = true,
            display_order = 1,
            conflicts_with = "lower_bound"
        )]
        original_flow: bool,

        /// Add [p]enalty arcs between vertices with non-zero supply/demand relationship
        #[arg(long, short = 'p', global = true, display_order = 1)]
        penalty_arcs: bool,

        /// Number of vertices
        vertices: usize,

        #[command(flatten)]
        random: RandomizationArgs,
    },
    /// Export the network vertices and arcs as a latex figure.
    Latex {
        /// Path to a file containing a network to be used as input.
        in_file: String,

        /// Where to save the output to.
        out_file: String,

        /// Disable vertex and arc labels. Useful for large networks.
        #[arg(long, display_order = 0)]
        no_text: bool,

        /// Width of the resulting tikz picture.
        #[arg(long, display_order = 0, default_value_t = 12.0)]
        width: f32,

        /// Enable marking of "station" vertices.
        #[arg(long, display_order = 0)]
        mark_stations: bool,
    },
}

fn parse_triplet(s: &str) -> Result<(usize, usize, usize), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err("Triplet must contain exactly three values".into());
    }
    let first = parts[0]
        .parse::<usize>()
        .map_err(|_| "Failed to parse first number")?;
    let second = parts[1]
        .parse::<usize>()
        .map_err(|_| "Failed to parse second number")?;
    let third = parts[2]
        .parse::<usize>()
        .map_err(|_| "Failed to parse third number")?;
    Ok((first, second, third))
}

fn parse_tuple(s: &str) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("Tuple must contain exactly two values".into());
    }
    let first = parts[0]
        .parse::<usize>()
        .map_err(|_| "Failed to parse first number")?;
    let second = parts[1]
        .parse::<usize>()
        .map_err(|_| "Failed to parse second number")?;
    Ok((first, second))
}
