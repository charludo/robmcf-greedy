#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro

use grb::prelude::*;

use crate::{matrix::Matrix, Network};

pub fn gurobi(network: &mut Network) -> Result<(), Box<dyn std::error::Error>> {
    let (balances, solution) = match &network.solution {
        Some(solution) => (&solution.supply_remaining, &solution.arc_loads),
        None => (
            &network.balances,
            &(0..network.balances.len())
                .map(|_| Matrix::filled_with(0, network.vertices.len(), network.vertices.len()))
                .collect(),
        ),
    };
    for (b, scenario) in balances.iter().enumerate() {
        let mut model = Model::new(&format!("scenario_{b}"))?;
        let capacities = &network.capacities.subtract(&solution[b]);

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
                    let commodity_supply_from_vertex = scenario.get(i_vertex, i_commodity);
                    let _ = model.add_constr(
                        &format!("flow_balance_{i_commodity}({i_vertex})"),
                        c!(outgoing_flow - incoming_flow == commodity_supply_from_vertex),
                    )?;
                } else {
                    let total_commodity_demand: i32 =
                        scenario.as_columns()[i_commodity].iter().sum::<usize>() as i32;
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
        model.write(&format!("scenario_{b}.lp"))?;

        // optimize the model
        model.optimize()?;
        assert_eq!(model.status()?, Status::Optimal);
        //
        // // Querying a model attribute
        // assert_eq!(model.get_attr(attr::ObjVal)?, 59.0);
        //
        // // Querying a model object attributes
        // assert_eq!(model.get_obj_attr(attr::Slack, &c0)?, -34.5);
        // let x1_name = model.get_obj_attr(attr::VarName, &x1)?;
        //
        // // Querying an attribute for multiple model objects
        // let val = model.get_obj_attr_batch(attr::X, vec![x1, x2])?;
        // assert_eq!(val, [6.5, 7.0]);
        //
        // // Querying variables by name
        // assert_eq!(model.get_var_by_name(&x1_name)?, Some(x1));
    }

    Ok(())
}
