use std::{
    collections::HashMap,
    sync::{Arc, Barrier},
};

use rayon::{iter::ParallelIterator, ThreadPoolBuilder};

use crate::{
    network::{AuxiliaryNetwork, Scenario, ScenarioSolution},
    Options, Result,
};

pub(crate) fn greedy(
    network: &mut AuxiliaryNetwork,
    options: &Options,
) -> Result<Vec<ScenarioSolution>> {
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
        let barrier_clone = barrier.clone();

        let _: Result<Vec<_>> = network
            .scenarios
            .par_iter_mut()
            .map(|mut entry| {
                let (_, scenario) = entry.pair_mut();
                scenario.refresh_relative_draws(&waiting_at_fixed_arcs, &options.relative_draw_fn);

                handle_free(scenario, &network.fixed_arcs)?;
                handle_fixed(
                    scenario,
                    network,
                    &consistent_flows_to_move,
                    exists_free_supply,
                )?;

                barrier_clone.wait();
                Ok(())
            })
            .collect();
    }

    get_solutions(network)
}

fn handle_free(scenario: &mut Scenario, fixed_arcs: &[usize]) -> Result<()> {
    let mut i = 0;
    while i < scenario.b_tuples_free.len() {
        let b_tuple = &mut scenario.b_tuples_free[i];
        let next_vertex = scenario.network_state.get_next_vertex(
            scenario.id,
            b_tuple.origin,
            b_tuple.s,
            b_tuple.t,
        )?;

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
            scenario
                .supply_remaining
                .decrement(b_tuple.origin, b_tuple.t);
            scenario.b_tuples_free.remove(i);
            continue;
        }

        if fixed_arcs.contains(&next_vertex) {
            scenario
                .b_tuples_fixed
                .entry(next_vertex)
                .or_default()
                .push(b_tuple.clone());
            scenario.b_tuples_free.remove(i);
        }

        i += 1;
    }

    Ok(())
}

fn handle_fixed(
    scenario: &mut Scenario,
    network: &AuxiliaryNetwork,
    consistent_flows_to_move: &HashMap<usize, usize>,
    exists_free_supply: bool,
) -> Result<()> {
    for fixed_arc in &network.fixed_arcs {
        let mut move_method = "conistently";
        let mut consistent_flow_to_move = *consistent_flows_to_move.get(fixed_arc).unwrap_or(&0);
        if consistent_flow_to_move == 0 && !exists_free_supply {
            consistent_flow_to_move = scenario.waiting_at(*fixed_arc);
            scenario.use_slack(consistent_flow_to_move)?;
            move_method = "inconsistently";
        };

        if consistent_flow_to_move == 0 {
            return Ok(());
        }

        log::info!(
            "Moving {} units of supply {} along the fixed arc {}",
            consistent_flow_to_move,
            move_method,
            network.fixed_arc_repr(*fixed_arc)
        );

        let mut consistently_moved_supply = scenario
            .b_tuples_fixed
            .entry(*fixed_arc)
            .or_default()
            .drain(0..consistent_flow_to_move)
            .collect::<Vec<_>>();

        let mut i = 0;
        while i < consistently_moved_supply.len() {
            let b_tuple = &mut consistently_moved_supply[i];
            let fixed_arc_terminal = network.get_fixed_arc_terminal(*fixed_arc)?;
            scenario
                .network_state
                .use_arc(*fixed_arc, fixed_arc_terminal);
            b_tuple.s = fixed_arc_terminal;
            if b_tuple.s == b_tuple.t {
                scenario
                    .supply_remaining
                    .decrement(b_tuple.origin, b_tuple.t);
                consistently_moved_supply.remove(i);
                continue;
            }

            i += 1;
        }
        scenario.b_tuples_free.extend(consistently_moved_supply)
    }

    Ok(())
}

fn get_solutions(network: &AuxiliaryNetwork) -> Result<Vec<ScenarioSolution>> {
    network
        .scenarios
        .iter()
        .map(|scenario| -> Result<ScenarioSolution> {
            Ok(ScenarioSolution {
                id: scenario.id,
                slack_total: scenario.slack,
                slack_remaining: scenario.slack - scenario.slack_used,
                supply_remaining: ScenarioSolution::supply_from_auxiliary(
                    &scenario.supply_remaining,
                    network.fixed_arcs.len(),
                ),
                arc_loads: ScenarioSolution::arc_loads_from_auxiliary(
                    &scenario.network_state.arc_loads,
                    &network.fixed_arcs,
                    &network.fixed_arcs_memory,
                )?,
            })
        })
        .collect::<Result<Vec<_>>>()
}
