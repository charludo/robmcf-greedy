use clap::Parser;
use log::LevelFilter;

use robmcf_greedy::{Network, Options};

mod util;
use util::*;

fn main() {
    let args = Args::parse();

    let log_level = if args.quiet {
        LevelFilter::Off
    } else if args.trace {
        LevelFilter::Trace
    } else if args.debug {
        LevelFilter::Debug
    } else {
        match args.command {
            Commands::Benchmark { .. } => LevelFilter::Error,
            _ => LevelFilter::Info,
        }
    };
    setup_logger(log_level);

    let options = Options {
        cost_fn: args.costs,
        delta_fn: args.delta,
        relative_draw_fn: args.draw,
        slack_fn: args.slack,
        remainder_solve_method: args.remainder,
    };

    let network = match &args.command {
        Commands::Random {
            output: _,
            vertices,
            random,
            ..
        } => Ok(Network::from_random(
            &options,
            *vertices,
            random.station_density,
            random.arc_density,
            random.supply_density_min,
            random.supply_density_max,
            random.scenarios,
            random.umin,
            random.umax,
            random.cmin,
            random.cmax,
            random.bmin,
            random.bmax,
            random.fixed,
            random.fixed_consecutive,
        )),
        Commands::Benchmark { file, .. } => Network::from_file(&options, file),
        Commands::Solve { file, .. } => Network::from_file(&options, file),
        Commands::Ilp { file } => Network::from_file(&options, file),
    };

    let mut network = match network {
        Ok(network) => network,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    let (output, lower_bound, penalty_arcs) = match &args.command {
        Commands::Benchmark { iterations, .. } => {
            attempt!(network.validate_network());
            run_benchmark(&network, *iterations);
            return;
        }
        Commands::Ilp { .. } => {
            network.options.remainder_solve_method = robmcf_greedy::RemainderSolveMethod::Ilp;
            attempt!(network.solve_full_ilp());
            println!("{}", network);
            return;
        }
        Commands::Random {
            output,
            lower_bound,
            penalty_arcs,
            ..
        } => (output, lower_bound, penalty_arcs),
        Commands::Solve {
            randomize_capacities,
            randomize_costs,
            randomize_scenarios,
            randomize_fixed_arcs,
            random,
            output,
            lower_bound,
            penalty_arcs,
            ..
        } => {
            if *randomize_capacities {
                network.randomize_capacities(random.arc_density, random.umin, random.umax);
            }
            if *randomize_costs {
                network.randomize_costs(random.cmin, random.cmax);
            }
            if *randomize_scenarios {
                network.randomize_scenarios(
                    random.scenarios,
                    random.supply_density_min,
                    random.supply_density_max,
                    random.bmin,
                    random.bmax,
                );
            }
            if *randomize_fixed_arcs {
                network.randomize_fixed_arcs(random.fixed, random.fixed_consecutive);
            }
            if let Some(file) = output {
                attempt!(network.serialize(file));
            }
            (output, lower_bound, penalty_arcs)
        }
    };

    attempt!(network.validate_network());
    if let Some(output) = output {
        attempt!(network.serialize(output));
    }
    if *penalty_arcs {
        attempt!(network.add_penalty_arcs());
        println!("{}", network);
    }
    if *lower_bound {
        attempt!(network.lower_bound());
    }
    attempt!(network.preprocess());
    attempt!(network.solve());
    attempt!(network.solve_remainder());
    // attempt!(network.validate_solution());

    println!("{}", network);
}
