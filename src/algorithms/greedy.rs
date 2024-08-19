use crate::network::AuxiliaryNetwork;

pub(crate) fn greedy(network: &mut AuxiliaryNetwork) {
    while network.exists_free_supply() {
        let global_waiting_at_fixed_arcs = network.waiting();

        for scenario in network.scenarios.iter_mut() {
            let scenario_waiting_at_fixed_arcs = scenario.waiting(&network.fixed_arcs);

            scenario.b_tuples_free.retain_mut(|b_t| {
                let (can_take_fixed_arc, fixed_arc) = b_t.closest_fixed_arc();
                let mut next_vertex = *b_t.successor_map.get(b_t.s, b_t.t);
                if can_take_fixed_arc {
                    let relative_draw = global_waiting_at_fixed_arcs.get(&fixed_arc).unwrap()
                        - scenario_waiting_at_fixed_arcs.get(&fixed_arc).unwrap();
                    let cost_via_direct_path = b_t.distance_map.get(b_t.s, b_t.t);
                    let cost_via_fixed_arc = b_t.distance_map.get(b_t.s, fixed_arc.0)
                        + network.costs.get(fixed_arc.0, fixed_arc.1)
                        + b_t.distance_map.get(fixed_arc.1, b_t.t);

                    if *cost_via_direct_path > cost_via_fixed_arc - relative_draw {
                        next_vertex = *b_t.successor_map.get(b_t.s, b_t.t);
                    }
                }
                log::debug!(
                    "Moving supply with destination {} via: ({}->{})",
                    b_t.t,
                    b_t.s,
                    next_vertex
                );
                let _ = scenario.arc_loads.increment(b_t.s, next_vertex);
                let remaining_capacity = scenario.capacities.decrement(b_t.s, next_vertex);
                if remaining_capacity == 0 {
                    //
                }

                b_t.s = next_vertex;

                if b_t.s == b_t.t {
                    return false;
                }

                if can_take_fixed_arc && b_t.s == fixed_arc.0 {
                    scenario
                        .b_tuples_fixed
                        .entry(fixed_arc)
                        .or_insert_with(Vec::new)
                        .push(b_t.clone());
                    return false;
                }

                true
            });
        }

        let consistent_flows_to_move = network.max_consistent_flows();
        network.fixed_arcs.iter().for_each(|fixed_arc| {
            let consistent_flow_to_move = consistent_flows_to_move.get(fixed_arc).unwrap();
            log::debug!(
                "Moving {} units of supply consistently along the fixed arc ({}->{})",
                consistent_flow_to_move,
                fixed_arc.0,
                fixed_arc.1
            );
            network.scenarios.iter_mut().for_each(|scenario| {
                let mut consistently_moved_supply = scenario
                    .b_tuples_fixed
                    .entry(*fixed_arc)
                    .or_insert_with(Vec::new)
                    .drain(0..*consistent_flow_to_move)
                    .collect::<Vec<_>>();
                consistently_moved_supply.retain_mut(|b_t| {
                    scenario.arc_loads.increment(fixed_arc.0, fixed_arc.1);
                    scenario.capacities.decrement(fixed_arc.0, fixed_arc.1);
                    b_t.mark_arc_used(&fixed_arc);
                    b_t.s = fixed_arc.1;
                    if b_t.s == b_t.t {
                        return false;
                    }

                    true
                });
                scenario.b_tuples_free.extend(consistently_moved_supply)
            });
        });
    }

    log::debug!(
        "Greedy found the following solution:\n{}",
        network
            .scenarios
            .iter()
            .map(|scenario| format!("{}", scenario.arc_loads))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
