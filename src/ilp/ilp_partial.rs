#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use super::util::*;
use grb::prelude::*;

use crate::{network::ScenarioSolution, Matrix, Network, Result, SolverError};

pub(crate) fn gurobi_partial(network: &mut Network) -> Result<Vec<ScenarioSolution>> {
    let mut state = match &network.solutions {
        Some(solutions) => solutions.clone(),
        None => network
            .balances
            .iter()
            .enumerate()
            .map(|(i, b)| ScenarioSolution::new(i, b))
            .collect::<Vec<_>>(),
    };
    for (lambda, scenario) in state.iter_mut().enumerate() {
        let env = match log::log_enabled!(log::Level::Debug) {
            true => Env::new("gurobi.log")?,
            false => get_quiet_env(),
        };

        let mut model = Model::with_env(&format!("scenario_{lambda}"), env)?;
        let capacities = &network.capacities.subtract(&scenario.arc_loads);

        let commodity_flows = get_vars(&mut model, network, capacities, lambda)?;
        let arc_loads = get_arc_loads(network, &commodity_flows);

        add_multi_commodity_flow_constraints(
            &mut model,
            &commodity_flows,
            &scenario.supply_remaining,
            lambda,
        )?;
        add_capacity_constraints(&mut model, network, capacities, &arc_loads, lambda)?;

        // Objective function
        let total_scenario_cost = network
            .costs
            .indices()
            .map(|(u, v)| arc_loads.get(u, v).clone() * *network.costs.get(u, v))
            .grb_sum();
        model.set_objective(total_scenario_cost, Minimize)?;
        model.write(&format!("scenario_{lambda}.lp"))?;

        model.optimize()?;
        match model.status()? {
            Status::Optimal => {}
            Status::SubOptimal => {}
            _ => return Err(SolverError::GurobiSolutionError(scenario.id)),
        }

        for commodity_flow in commodity_flows.elements() {
            let result = model.get_obj_attr_batch(attr::X, commodity_flow.elements().copied())?;
            scenario.arc_loads = scenario.arc_loads.add(&Matrix::from_elements(
                result
                    .iter()
                    .map(|e| *e as usize)
                    .collect::<Vec<_>>()
                    .as_slice(),
                scenario.arc_loads.num_rows(),
                scenario.arc_loads.num_columns(),
            ));
            scenario.supply_remaining = Matrix::filled_with(
                0,
                scenario.supply_remaining.num_rows(),
                scenario.supply_remaining.num_columns(),
            );
        }
    }

    Ok(state)
}
