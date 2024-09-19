#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use grb::prelude::*;

use crate::{
    auxiliary::generate_intermediate_arc_sets, network::ScenarioSolution, Matrix, Network, Result,
    SolverError,
};

use super::floyd_warshall;

pub fn gurobi_full(network: &mut Network) -> Result<Vec<ScenarioSolution>> {
    let (dist, _) = floyd_warshall(&network.capacities, &network.costs);
    let arc_sets = generate_intermediate_arc_sets(
        &dist,
        &network.costs,
        &network.capacities,
        &network.options.delta_fn,
    );
    let slack_values = network.options.slack_fn.apply(&network.balances);
    log::error!("{:?}", slack_values);

    let env = match log::log_enabled!(log::Level::Debug) {
        true => Env::new("gurobi.log")?,
        false => {
            let mut env = Env::empty().unwrap();
            env.set(param::OutputFlag, 0).unwrap();
            env.start().unwrap()
        }
    };

    let mut model = Model::with_env("network", env)?;

    let mut scenario_flows = Vec::new();
    let mut scenario_commodity_flows = Vec::new();
    let mut scenario_slack = Vec::new();
    for (lambda, balance) in network.balances.iter().enumerate() {
        let mut commodity_flows: Matrix<Matrix<Var>> = Matrix::filled_with(
            Matrix::empty(),
            network.vertices.len(),
            network.vertices.len(),
        );
        for (s, t) in commodity_flows.indices() {
            // Generate variables
            let mut s_t_flows = Vec::new();
            for (u, v) in network.capacities.indices() {
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
                        c!(outgoing_flow - incoming_flow == *balance.get(s, t)),
                    )?;
                } else if vertex == t {
                    let _ = model.add_constr(
                        &format!("flow_balance^{lambda}_({s}{t})({vertex})"),
                        c!(incoming_flow - outgoing_flow == *balance.get(s, t)),
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
        for (u, v) in network.capacities.indices() {
            // Fixed arcs have unlimited capacity
            if network.fixed_arcs.contains(&(u, v)) {
                continue;
            }
            let _ = model.add_constr(
                &format!("capacity^{lambda}_({u},{v})"),
                c!(arc_loads.get(u, v).clone() <= *network.capacities.get(u, v)),
            )?;
        }

        // Total slack constraints
        let mut slack_variables = Vec::new();
        for (a_0, a_1) in network.fixed_arcs.iter() {
            slack_variables.push(add_intvar!(model, name: &format!("slack^{lambda}_({a_0},{a_1})"), bounds: 0..slack_values[lambda])?);
        }
        let _ = model.add_constr(
            &format!("total_slack^{lambda}"),
            c!(slack_variables.clone().grb_sum() <= slack_values[lambda]),
        )?;

        scenario_flows.push(arc_loads);
        scenario_commodity_flows.push(commodity_flows);
        scenario_slack.push(slack_variables);
    }

    // Consistent flow constraints
    for (fixed_arc, (a_0, a_1)) in network.fixed_arcs.iter().enumerate() {
        for lambda_0 in 0..scenario_flows.len() {
            for lambda_1 in lambda_0 + 1..scenario_flows.len() {
                let _ = model.add_constr(
                    &format!("consistent_flow_({a_0},{a_1})"),
                    c!(scenario_flows[lambda_0].get(*a_0, *a_1).clone()
                        + scenario_slack[lambda_0][fixed_arc]
                        == (scenario_flows[lambda_1].get(*a_0, *a_1).clone())
                            + scenario_slack[lambda_1][fixed_arc]),
                )?;
            }
        }
    }

    // Helper variable for minimizing network cost
    let c_max = add_intvar!(model, name: "max_scenario_cost", bounds: 0..)?;

    // Scenario cost constraints
    for (lambda, scenario_flow) in scenario_flows.iter().enumerate() {
        let scenario_cost = network
            .costs
            .indices()
            .map(|(u, v)| scenario_flow.get(u, v).clone() * *network.costs.get(u, v))
            .grb_sum();
        let _ = model.add_constr(
            &format!("scenario_cost_{lambda}"),
            c!(scenario_cost <= c_max),
        )?;
    }

    // Objective function
    model.set_objective(c_max, Minimize)?;
    model.write("network.lp")?;

    model.optimize()?;
    match model.status()? {
        Status::Optimal => {}
        Status::SubOptimal => {}
        _ => return Err(SolverError::GurobiSolutionError(0)),
    }

    let mut scenario_arc_loads = Vec::new();
    for scenario_flow in scenario_commodity_flows {
        let mut arc_loads = Matrix::filled_with(0, network.vertices.len(), network.vertices.len());
        for s_t_flow in scenario_flow.elements() {
            let result = model.get_obj_attr_batch(attr::X, s_t_flow.elements().copied())?;
            arc_loads = arc_loads.add(&Matrix::from_elements(
                result
                    .iter()
                    .map(|e| *e as usize)
                    .collect::<Vec<_>>()
                    .as_slice(),
                arc_loads.num_rows(),
                arc_loads.num_columns(),
            ));
        }
        scenario_arc_loads.push(arc_loads);
    }

    let mut solutions = Vec::new();
    for i in 0..scenario_arc_loads.len() {
        let mut slack = 0;
        for (a_0, a_1) in &network.fixed_arcs {
            let consistent_flow = scenario_arc_loads
                .iter()
                .map(|arc_loads| *arc_loads.get(*a_0, *a_1))
                .min()
                .unwrap_or(0);
            slack += *scenario_arc_loads[i].get(*a_0, *a_1) - consistent_flow;
        }

        solutions.push(ScenarioSolution {
            id: i,
            slack_total: slack_values[i],
            slack_remaining: slack_values[i] - slack,
            supply_remaining: Matrix::filled_with(
                0,
                network.vertices.len(),
                network.vertices.len(),
            ),
            arc_loads: scenario_arc_loads[i].clone(),
        })
    }

    Ok(solutions)
}
