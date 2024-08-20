use crate::network::AuxiliaryNetwork;

pub(crate) fn greedy(network: &mut AuxiliaryNetwork) {
    while network.exists_free_supply() {
        let global_waiting_at_fixed_arcs = network.waiting();

        for scenario in network.scenarios.iter_mut() {
            let scenario_waiting_at_fixed_arcs = scenario.waiting(&network.fixed_arcs);

            let mut b_tuples = std::mem::take(&mut scenario.b_tuples_free);
            b_tuples.retain_mut(|b_tuple| {
                let (next_vertex, fixed_arc) = b_tuple.get_next_vertex(
                    &global_waiting_at_fixed_arcs,
                    &scenario_waiting_at_fixed_arcs,
                    &network.costs,
                );

                let distance_maps_need_refresh = scenario.use_arc(b_tuple.s, next_vertex);
                if distance_maps_need_refresh {
                    scenario.refresh_maps(b_tuple.s, next_vertex, &network.costs);
                }

                b_tuple.s = next_vertex;

                if b_tuple.s == b_tuple.t {
                    return false;
                }

                if fixed_arc.is_some() {
                    scenario
                        .b_tuples_fixed
                        .entry(fixed_arc.unwrap())
                        .or_insert_with(Vec::new)
                        .push(b_tuple.clone());
                    return false;
                }

                true
            });
            scenario.b_tuples_free = b_tuples;
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
                consistently_moved_supply.retain_mut(|b_tuple| {
                    scenario.arc_loads.increment(fixed_arc.0, fixed_arc.1);
                    scenario.capacities.decrement(fixed_arc.0, fixed_arc.1);
                    b_tuple.mark_arc_used(&fixed_arc);
                    b_tuple.s = fixed_arc.1;
                    if b_tuple.s == b_tuple.t {
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
