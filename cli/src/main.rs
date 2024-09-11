use clap::Parser;
use log::LevelFilter;

use robmcf_greedy::{Network, Options};

mod util;
use util::*;

fn main() {
    let args = Args::parse();

    let log_level = if args.quiet {
        LevelFilter::Off
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
            fixed,
            arc_density,
            umin,
            umax,
            cmin,
            cmax,
            scenarios,
            balance_density,
            bmin,
            bmax,
        } => Ok(Network::from_random(
            &options,
            *vertices,
            *arc_density,
            *balance_density,
            *scenarios,
            (*bmin, *bmax),
            (*umin, *umax),
            (*cmin, *cmax),
            *fixed,
        )),
        Commands::Benchmark { file, .. } => Network::from_file(&options, file),
        Commands::Solve { file, .. } => Network::from_file(&options, file),
    };

    let mut network = match network {
        Ok(network) => network,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    attempt!(network.validate_network());

    match &args.command {
        Commands::Benchmark { iterations, .. } => {
            run_benchmark(&network, *iterations);
            return;
        }
        Commands::Random { output, .. } => {
            if let Some(file) = output {
                attempt!(network.serialize(file));
            }
        }
        Commands::Solve { .. } => {}
    }

    if !&args.skip_baseline {
        attempt!(network.lower_bound());
    }
    attempt!(network.preprocess());
    attempt!(network.solve());
    attempt!(network.solve_remainder());
    attempt!(network.validate_solution());

    println!("{}", network);
}
