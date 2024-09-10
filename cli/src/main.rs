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

    let mut network = match &args.command {
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
        } => Network::from_random(
            &options,
            *vertices,
            *arc_density,
            *balance_density,
            *scenarios,
            (*bmin, *bmax),
            (*umin, *umax),
            (*cmin, *cmax),
            *fixed,
        ),
        Commands::Benchmark { file, .. } => Network::from_file(&options, file),
        Commands::Solve { file, .. } => Network::from_file(&options, file),
    };

    network.validate_network();
    match &args.command {
        Commands::Benchmark { iterations, .. } => run_benchmark(&network, *iterations),
        Commands::Random { output, .. } => {
            network.preprocess();
            if let Some(file) = output {
                network.serialize(file);
            }
            network.solve();
            network.solve_remainder();
            network.validate_solution();
            println!("{}", network);
        }
        Commands::Solve { .. } => {
            network.preprocess();
            network.solve();
            network.solve_remainder();
            network.validate_solution();
            println!("{}", network);
        }
    }
}
