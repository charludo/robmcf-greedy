use std::sync::{Arc, Barrier};

use rayon::{
    iter::{IntoParallelRefMutIterator, ParallelIterator},
    ThreadPoolBuilder,
};

use crate::network::AuxiliaryNetwork;

pub(crate) fn greedy(network: &mut AuxiliaryNetwork) {
    let num_threads = network.scenarios.len();
    let barrier = Arc::new(Barrier::new(num_threads));
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    while network.exists_supply() {
        let exists_free_supply = network.exists_free_supply();
        let global_waiting_at_fixed_arcs = network.waiting();
        let consistent_flows_to_move = network.max_consistent_flows();
        let barrier_clone = barrier.clone();

        network.scenarios.par_iter_mut().for_each(|scenario| {
            let scenario_waiting_at_fixed_arcs = scenario.waiting(&network.fixed_arcs);

            let mut b_tuples = std::mem::take(&mut scenario.b_tuples_free);
            b_tuples.retain_mut(|b_tuple| {
                let (next_vertex, fixed_arc) = b_tuple.get_next_vertex(
                    &global_waiting_at_fixed_arcs,
                    &scenario_waiting_at_fixed_arcs,
                    &network.costs,
                );

                scenario.use_arc(b_tuple.s, next_vertex, &network.costs);

                log::debug!(
                    "Moving supply with destination {} via: ({}->{})",
                    b_tuple.t,
                    b_tuple.s,
                    next_vertex
                );
                b_tuple.s = next_vertex;

                if b_tuple.s == b_tuple.t {
                    return false;
                }

                if let Some(fixed_arc) = fixed_arc {
                    scenario
                        .b_tuples_fixed
                        .entry(fixed_arc)
                        .or_default()
                        .push(b_tuple.clone());
                    return false;
                }

                true
            });
            scenario.b_tuples_free = b_tuples;

            network.fixed_arcs.iter().for_each(|fixed_arc| {
                let consistent_flow_to_move = if exists_free_supply {
                    *consistent_flows_to_move.get(fixed_arc).unwrap()
                } else {
                    let inconsistent_flow = scenario.waiting_at(fixed_arc);
                    scenario.slack += inconsistent_flow;
                    inconsistent_flow
                };

                if consistent_flow_to_move == 0 {
                    return;
                }

                log::info!(
                    "Moving {} units of supply consistently along the fixed arc ({}->{})",
                    consistent_flow_to_move,
                    fixed_arc.0,
                    fixed_arc.1
                );

                let mut consistently_moved_supply = scenario
                    .b_tuples_fixed
                    .entry(*fixed_arc)
                    .or_default()
                    .drain(0..consistent_flow_to_move)
                    .collect::<Vec<_>>();

                consistently_moved_supply.retain_mut(|b_tuple| {
                    scenario.use_arc(fixed_arc.0, fixed_arc.1, &network.costs);
                    b_tuple.mark_arc_used(fixed_arc);
                    b_tuple.s = fixed_arc.1;
                    if b_tuple.s == b_tuple.t {
                        return false;
                    }

                    true
                });
                scenario.b_tuples_free.extend(consistently_moved_supply)
            });

            barrier_clone.wait();
        });
    }
}
