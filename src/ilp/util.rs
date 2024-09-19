#![allow(clippy::useless_conversion)] // Clippy doesn't like the "c!()" constraints macro
                                      //
use crate::{
    algorithms::floyd_warshall, auxiliary::generate_intermediate_arc_sets, matrix::Matrix,
    DeltaFunction, Network, Result,
};
use grb::prelude::*;

pub(super) fn get_quiet_env() -> Env {
    let mut env = Env::empty().unwrap();
    env.set(grb::param::OutputFlag, 0).unwrap();
    env.start().unwrap()
}

pub(super) fn get_arc_sets(
    capacities: &Matrix<usize>,
    costs: &Matrix<usize>,
    delta_fn: &DeltaFunction,
) -> Matrix<Matrix<bool>> {
    let (dist, _) = floyd_warshall(capacities, costs);
    generate_intermediate_arc_sets(&dist, costs, capacities, delta_fn)
}

pub(super) fn get_vars(
    model: &mut Model,
    network: &Network,
    capacities: &Matrix<usize>,
    lambda: usize,
) -> Result<Matrix<Matrix<Var>>> {
    let arc_sets = get_arc_sets(capacities, &network.costs, &network.options.delta_fn);
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

        commodity_flows.set(s, t, s_t_flows);
    }
    Ok(commodity_flows)
}

pub(super) fn get_arc_loads(
    network: &Network,
    commodity_flows: &Matrix<Matrix<Var>>,
) -> Matrix<Expr> {
    Matrix::from_elements(
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
    )
}

pub(super) fn add_multi_commodity_flow_constraints(
    model: &mut Model,
    commodity_flows: &Matrix<Matrix<Var>>,
    balance: &Matrix<usize>,
    lambda: usize,
) -> Result<()> {
    for (s, t) in commodity_flows.indices() {
        let s_t_flows = commodity_flows.get(s, t);
        for vertex in 0..commodity_flows.num_rows() {
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
    }
    Ok(())
}

pub(super) fn add_capacity_constraints(
    model: &mut Model,
    network: &Network,
    capacities: &Matrix<usize>,
    arc_loads: &Matrix<Expr>,
    lambda: usize,
) -> Result<()> {
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
    Ok(())
}
