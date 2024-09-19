#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use super::util::*;
use grb::prelude::*;

use crate::{network::ScenarioSolution, Matrix, Network, Result, SolverError};

pub(crate) fn gurobi_full(network: &mut Network) -> Result<Vec<ScenarioSolution>> {
    let slack_values = network.options.slack_fn.apply(&network.balances);

    let env = match log::log_enabled!(log::Level::Debug) {
        true => Env::new("gurobi.log")?,
        false => get_quiet_env(),
    };

    let mut model = Model::with_env("network", env)?;

    let mut scenario_flows = Vec::new();
    let mut scenario_commodity_flows = Vec::new();
    let mut scenario_slack = Vec::new();

    for (lambda, balance) in network.balances.iter().enumerate() {
        let commodity_flows = get_vars(&mut model, network, &network.capacities, lambda)?;
        let arc_loads = get_arc_loads(network, &commodity_flows);

        add_multi_commodity_flow_constraints(&mut model, &commodity_flows, balance, lambda)?;
        add_capacity_constraints(&mut model, network, &network.capacities, &arc_loads, lambda)?;

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
