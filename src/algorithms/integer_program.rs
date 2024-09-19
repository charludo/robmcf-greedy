#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use grb::prelude::*;

use crate::{
    auxiliary::generate_intermediate_arc_sets, network::ScenarioSolution, Matrix, Network, Result,
    SolverError,
};

use super::floyd_warshall;

pub fn gurobi(network: &mut Network) -> Result<Vec<ScenarioSolution>> {
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
            false => {
                let mut env = Env::empty().unwrap();
                env.set(param::OutputFlag, 0).unwrap();
                env.start().unwrap()
            }
        };

        let mut model = Model::with_env(&format!("scenario_{lambda}"), env)?;
        let capacities = &network.capacities.subtract(&scenario.arc_loads);
        let (dist, _) = floyd_warshall(capacities, &network.costs);
        let arc_sets = generate_intermediate_arc_sets(
            &dist,
            &network.costs,
            capacities,
            &network.options.delta_fn,
        );

        let mut commodity_flows: Matrix<Matrix<Var>> = Matrix::filled_with(
            Matrix::empty(),
            network.vertices.len(),
            network.vertices.len(),
        );
        for (s, t) in commodity_flows.indices() {
            // Generate variables
            let mut s_t_flows = Vec::new();
            for (u, v) in capacities.indices() {
                let upper_bound = if *arc_sets.get(s, t).get(u, v) {
                    // Fixed arcs have unlimited capacity
                    if network.fixed_arcs.contains(&(u, v)) {
                        usize::MAX
                    } else {
                        *network.capacities.get(u, v)
                    }
                } else {
                    0
                };
                // Combines non-negative, capacity, and intermediate arc set bounds
                s_t_flows.push(add_intvar!(model, name: &format!("f^{lambda}_({s},{t})(({u},{v}))"), bounds: 0..upper_bound)?);
            }

            let s_t_flows =
                Matrix::from_elements(&s_t_flows, network.vertices.len(), network.vertices.len());

            // (s, t) flow balance constraints
            for vertex in 0..network.vertices.len() {
                let outgoing_flow = s_t_flows.as_rows()[vertex].clone().grb_sum();
                let incoming_flow = s_t_flows.as_columns()[vertex].clone().grb_sum();

                if vertex == s {
                    let _ = model.add_constr(
                        &format!("flow_balance^{lambda}_({s}{t})({vertex})"),
                        c!(outgoing_flow - incoming_flow == *scenario.supply_remaining.get(s, t)),
                    )?;
                } else if vertex == t {
                    let _ = model.add_constr(
                        &format!("flow_balance^{lambda}_({s}{t})({vertex})"),
                        c!(incoming_flow - outgoing_flow == *scenario.supply_remaining.get(s, t)),
                    )?;
                } else {
                    let _ = model.add_constr(
                        &format!("flow_balance^{lambda}_({s}{t})({vertex})"),
                        c!(outgoing_flow - incoming_flow == 0),
                    )?;
                }
            }

            commodity_flows.set(s, t, s_t_flows);
        }

        // Pre-compute expressions for the total flow along each arc
        let arc_loads = Matrix::from_elements(
            network
                .capacities
                .indices()
                .map(|(u, v)| {
                    commodity_flows
                        .elements()
                        .map(|c_f| c_f.get(u, v))
                        .grb_sum()
                })
                .collect::<Vec<_>>()
                .as_slice(),
            network.vertices.len(),
            network.vertices.len(),
        );

        // Capacity constraints
        for (u, v) in capacities.indices() {
            // Fixed arcs have unlimited capacity
            if network.fixed_arcs.contains(&(u, v)) {
                continue;
            }
            let _ = model.add_constr(
                &format!("capacity^{lambda}_({u},{v})"),
                c!(arc_loads.get(u, v).clone() <= *capacities.get(u, v)),
            )?;
        }

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
