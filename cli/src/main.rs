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
        Commands::Latex { in_file, .. } => Network::from_file(&options, in_file),
    };

    let mut network = match network {
        Ok(network) => network,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    let (output, lower_bound, original_flow, penalty_arcs, fix_best) = match &args.command {
        Commands::Benchmark { iterations, .. } => {
            attempt!(network.validate_network());
            let (network, time_preprocess, time_solve) = run_benchmark(&network, *iterations);
            if let Some(export) = args.export {
                attempt!(network.export(&export, Some(time_preprocess), Some(time_solve)));
            };
            return;
        }
        Commands::Latex {
            out_file,
            no_text,
            width,
            mark_stations,
            ..
        } => {
            attempt!(network.to_latex(out_file, *no_text, *width, *mark_stations));
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
            original_flow,
            penalty_arcs,
            random,
            ..
        } => (
            output,
            lower_bound,
            original_flow,
            penalty_arcs,
            random.fix_best,
        ),
        Commands::Solve {
            randomize_capacities,
            randomize_costs,
            randomize_scenarios,
            randomize_fixed_arcs,
            random,
            output,
            lower_bound,
            original_flow,
            penalty_arcs,
            override_fixed,
            override_costs,
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
            if let Some(r#override) = override_fixed {
                let other_network = Network::from_file(&options, r#override);
                match other_network {
                    Ok(o_n) => {
                        network.fixed_arcs = o_n.fixed_arcs;
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
            if let Some(r#override) = override_costs {
                for &(s, t, c) in r#override.iter() {
                    if s >= network.vertices.len() || t >= network.vertices.len() {
                        log::error!(
                            "{}",
                            robmcf_greedy::SolverError::NetworkShapeError(format!(
                                "Attempted to set cost for arc at indices ({},{}), but only {} vertices exist.",
                                s,
                                t,
                                network.vertices.len()
                            ))
                        );
                        return;
                    }

                    network.costs.set(s, t, c);
                }
            }
            if let Some(file) = output {
                attempt!(network.serialize(file));
            }
            (
                output,
                lower_bound,
                original_flow,
                penalty_arcs,
                random.fix_best,
            )
        }
    };

    attempt!(network.validate_network());
    if let Some(output) = output {
        attempt!(network.serialize(output));
    }
    if *penalty_arcs {
        attempt!(network.add_penalty_arcs());
    }
    if *lower_bound {
        attempt!(network.lower_bound());
    }
    if *original_flow {
        attempt!(network.original_flow());
    }
    if let Some(number) = fix_best {
        attempt!(network.fix_best_candidates(number));
    }
    attempt!(network.preprocess());
    attempt!(network.solve());
    attempt!(network.solve_remainder());
    attempt!(network.validate_solution());
    if let Some(output) = output {
        // Second time to also capture the baseline/solution
        attempt!(network.serialize(output));
    }
    if let Some(export) = args.export {
        attempt!(network.export(&export, None, None));
    };

    println!("{}", network);
}
