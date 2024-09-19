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
                        let next_vertex = scenario.network_state.get_next_vertex(
                            scenario.id,
                            token.origin,
                            token.s,
                            token.t,
                        )?;

                        log::debug!(
                            "Moving supply in scenario {} with origin {} and destination {} via: ({}->{})",
                            scenario.id,
                            token.origin,
                            token.t,
                            token.s,
                            next_vertex
                        );

                        scenario.network_state.use_arc(token.s, next_vertex);
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
    network
        .scenarios
        .iter()
        .map(|scenario| -> Result<ScenarioSolution> {
            Ok(ScenarioSolution {
                id: scenario.id,
                slack_total: scenario.slack,
                slack_remaining: scenario.slack - scenario.slack_used,
                supply_remaining: scenario.supply_remaining.clone(),
                arc_loads: scenario.network_state.arc_loads.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()
}
