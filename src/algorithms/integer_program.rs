#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use grb::prelude::*;

use crate::{matrix::Matrix, network::ScenarioSolution, Network, Result, SolverError};

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
    for (i_scenario, scenario) in state.iter_mut().enumerate() {
        let env = match log::log_enabled!(log::Level::Debug) {
            true => Env::new("gurobi.log")?,
            false => {
                let mut env = Env::empty().unwrap();
                env.set(param::OutputFlag, 0).unwrap();
                env.start().unwrap()
            }
        };

        let mut model = Model::with_env(&format!("scenario_{i_scenario}"), env)?;
        let capacities = &network.capacities.subtract(&scenario.arc_loads);

        let mut commodity_flows = Vec::new();
        for (i_commodity, commodity) in network.vertices.iter().enumerate() {
            let mut rows = Vec::new();
            for (i, u_row) in capacities.rows_iter().enumerate() {
                let mut row = Vec::new();
                for (j, u) in u_row.enumerate() {
                    // Add a variable for every arc. The bounds already encode the capacity constraints
                    let var = add_intvar!(model, name: &format!("f_{}(({i},{j}))", commodity.name), bounds: 0..*u)?;
                    row.push(var);
                }
                rows.push(row);
            }
            let vars = Matrix::from_rows(&rows);

            // Multi-Commodity flow balance constraint
            for i_vertex in 0..network.vertices.len() {
                let outgoing_flow = vars.as_rows()[i_vertex].clone().grb_sum();
                let incoming_flow = vars.as_columns()[i_vertex].clone().grb_sum();

                if i_commodity != i_vertex {
                    let commodity_supply_from_vertex =
                        scenario.supply_remaining.get(i_vertex, i_commodity);
                    let _ = model.add_constr(
                        &format!("flow_balance_{i_commodity}({i_vertex})"),
                        c!(outgoing_flow - incoming_flow == commodity_supply_from_vertex),
                    )?;
                } else {
                    let total_commodity_demand: i32 =
                        scenario.supply_remaining.as_columns()[i_commodity]
                            .iter()
                            .sum::<usize>() as i32;
                    let _ = model.add_constr(
                        &format!("flow_balance_{i_commodity}"),
                        c!(outgoing_flow - incoming_flow == -total_commodity_demand),
                    )?;
                }
            }

            commodity_flows.push(vars);
        }

        // Pre-compute expressions for the total flow along each arc
        let commodity_loads = Matrix::from_elements(
            capacities
                .indices()
                .map(|(s, t)| commodity_flows.iter().map(|c_f| c_f.get(s, t)).grb_sum())
                .collect::<Vec<_>>()
                .as_slice(),
            network.vertices.len(),
            network.vertices.len(),
        );

        // Capacity constraints
        for (s, t) in capacities.indices() {
            let _ = model.add_constr(
                &format!("capacity_({s},{t})"),
                c!(commodity_loads.get(s, t).clone() <= *capacities.get(s, t)),
            )?;
        }

        // Objective function
        let total_scenario_cost = network
            .costs
            .indices()
            .map(|(s, t)| commodity_loads.get(s, t).clone() * *network.costs.get(s, t))
            .grb_sum();
        model.set_objective(total_scenario_cost, Minimize)?;
        model.write(&format!("scenario_{i_scenario}.lp"))?;

        model.optimize()?;
        match model.status()? {
            Status::Optimal => {}
            Status::SubOptimal => {}
            _ => return Err(SolverError::GurobiSolutionError(scenario.id)),
        }
        for commodity_flow in &commodity_flows {
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
