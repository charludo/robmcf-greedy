use std::sync::{Arc, Barrier};

use rayon::{iter::ParallelIterator, ThreadPoolBuilder};

use crate::{network::AuxiliaryNetwork, Options};

pub(crate) fn greedy(network: &mut AuxiliaryNetwork, options: &Options) {
    let num_threads = network.scenarios.len();
    let barrier = Arc::new(Barrier::new(num_threads));
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    while network.exists_supply() {
        let exists_free_supply = network.exists_free_supply();
        let waiting_at_fixed_arcs = network.waiting();
        let consistent_flows_to_move = network.max_consistent_flows();

        let scenarios = std::mem::take(&mut network.scenarios);
        let barrier_clone = barrier.clone();

        scenarios.par_iter_mut().for_each(|mut entry| {
            let (_, scenario) = entry.pair_mut();
            let mut remaining_suppply = std::mem::take(&mut scenario.remaining_supply);
            scenario.refresh_relative_draws(&waiting_at_fixed_arcs, &options.relative_draw_fn);

            scenario.b_tuples_free.retain_mut(|b_tuple| {
                let next_vertex =
                    scenario
                        .network_state
                        .get_next_vertex(b_tuple.origin, b_tuple.s, b_tuple.t);

                if next_vertex == usize::MAX {
                    panic!("No feasible flow could be found!");
                }
                log::debug!(
                    "Moving supply in scenario {} with origin {} and destination {} via: ({}->{})",
                    scenario.id,
                    b_tuple.origin,
                    b_tuple.t,
                    b_tuple.s,
                    next_vertex
                );

                scenario.network_state.use_arc(b_tuple.s, next_vertex);
                b_tuple.s = next_vertex;

                if b_tuple.s == b_tuple.t {
                    remaining_suppply.decrement(b_tuple.origin, b_tuple.t);
                    return false;
                }

                if network.fixed_arcs.contains(&next_vertex) {
                    scenario
                        .b_tuples_fixed
                        .entry(next_vertex)
                        .or_default()
                        .push(b_tuple.clone());
                    return false;
                }

                true
            });

            network.fixed_arcs.iter().for_each(|fixed_arc| {
                let mut consistent_flow_to_move =
                    *consistent_flows_to_move.get(fixed_arc).unwrap_or(&0);
                if consistent_flow_to_move == 0 && !exists_free_supply {
                    consistent_flow_to_move = scenario.waiting_at(*fixed_arc);
                    scenario.use_slack(consistent_flow_to_move);
                };

                if consistent_flow_to_move == 0 {
                    return;
                }

                log::info!(
                    "Moving {} units of supply consistently along the fixed arc {}",
                    consistent_flow_to_move,
                    network.fixed_arc_repr(*fixed_arc)
                );

                let mut consistently_moved_supply = scenario
                    .b_tuples_fixed
                    .entry(*fixed_arc)
                    .or_default()
                    .drain(0..consistent_flow_to_move)
                    .collect::<Vec<_>>();

                consistently_moved_supply.retain_mut(|b_tuple| {
                    let fixed_arc_terminal = network.get_fixed_arc_terminal(*fixed_arc);
                    scenario
                        .network_state
                        .use_arc(*fixed_arc, fixed_arc_terminal);
                    b_tuple.s = fixed_arc_terminal;
                    if b_tuple.s == b_tuple.t {
                        remaining_suppply.decrement(b_tuple.origin, b_tuple.t);
                        return false;
                    }

                    true
                });
                scenario.b_tuples_free.extend(consistently_moved_supply)
            });

            scenario.remaining_supply = remaining_suppply;
            barrier_clone.wait();
        });
        network.scenarios = scenarios;
    }
}
