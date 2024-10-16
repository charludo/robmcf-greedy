use rayon::{iter::ParallelIterator, ThreadPoolBuilder};

use crate::{
    network::{AuxiliaryNetwork, ScenarioSolution},
    Options, Result,
};

pub(crate) fn greedy(
    network: &mut AuxiliaryNetwork,
    options: &Options,
) -> Result<Vec<ScenarioSolution>> {
    let pool = ThreadPoolBuilder::new().build().unwrap();
    let result: Result<()> = pool.install(|| {
        while network.exists_supply() {
            let fixed_arc_loads = network.snapshot_fixed_arc_loads();
            let result: Result<Vec<_>> = network
                .scenarios
                .par_iter_mut()
                .map(|mut entry| {
                    let (_, scenario) = entry.pair_mut();
                    scenario.refresh_relative_draws(&fixed_arc_loads, &options.relative_draw_fn);
                    let mut i = 0;
                    while i < scenario.supply_tokens.len() {
                        let token = &mut scenario.supply_tokens[i];
                        let next_vertex = scenario.network_state.get_next_vertex(token)?;

                        log::debug!(
                            "({}): Moving supply token {} via: ({}->{})",
                            scenario.id,
                            token,
                            token.s,
                            next_vertex
                        );

                        scenario.network_state.use_arc(token, next_vertex);
                        token.s = next_vertex;

                        if token.s == token.t {
                            scenario.supply_remaining.decrement(token.origin, token.t);
                            scenario.supply_tokens.remove(i);
                            continue;
                        }

                        i += 1;
                    }
                    Ok(())
                })
                .collect();
            result?;
        }
        Ok(())
    });
    result?;

    get_solutions(network)
}

fn get_solutions(network: &AuxiliaryNetwork) -> Result<Vec<ScenarioSolution>> {
    let total_consistent_flows: usize = network
        .fixed_arcs
        .iter()
        .map(|(a_0, a_1)| {
            network
                .scenarios
                .iter()
                .map(|scenario| *scenario.network_state.arc_loads.get(*a_0, *a_1))
                .max()
                .unwrap_or(0)
        })
        .sum();
    network
        .scenarios
        .iter()
        .map(|scenario| -> Result<ScenarioSolution> {
            let slack: usize = total_consistent_flows.saturating_sub(
                network
                    .fixed_arcs
                    .iter()
                    .map(|(a_0, a_1)| *scenario.network_state.arc_loads.get(*a_0, *a_1))
                    .sum::<usize>(),
            );
            Ok(ScenarioSolution {
                id: scenario.id,
                slack,
                supply_remaining: scenario.supply_remaining.clone(),
                arc_loads: scenario.network_state.arc_loads.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()
}
