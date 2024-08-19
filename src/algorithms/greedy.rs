use crate::network::AuxiliaryNetwork;

fn greedy(network: &mut AuxiliaryNetwork) {
    while network.exists_free_supply() {
        let waiting_at_fixed_arcs = network.waiting();

        for scenario in network.scenarios.iter_mut() {
            let fixed_arc = scenario.closest_fixed_arc(&network.fixed_arcs);
            let relative_draw =
                waiting_at_fixed_arcs.get(&fixed_arc).unwrap() - scenario.waiting_at(&fixed_arc);

            scenario.b_tuples_free.retain_mut(|b_t| {
                let cost_via_direct_path = scenario.distance_map.get(b_t.s, b_t.t);
                let cost_via_fixed_arc = scenario.distance_map.get(b_t.s, fixed_arc.0)
                    + network.costs.get(fixed_arc.0, fixed_arc.1)
                    + scenario.distance_map.get(fixed_arc.1, b_t.t);

                if *cost_via_direct_path < cost_via_fixed_arc - relative_draw {
                    let next_vertex_via_direct_path = scenario.successor_map.get(b_t.s, b_t.t);
                    b_t.s = *next_vertex_via_direct_path;
                } else {
                    let next_vertex_via_fixed_arc = scenario.successor_map.get(b_t.s, fixed_arc.0);
                    b_t.s = *next_vertex_via_fixed_arc;
                }

                if b_t.s == b_t.t {
                    return false;
                }

                if b_t.s == fixed_arc.0 {
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
            network.scenarios.iter_mut().for_each(|scenario| {
                let mut consistently_moved_supply = scenario
                    .b_tuples_fixed
                    .entry(*fixed_arc)
                    .or_insert_with(Vec::new)
                    .drain(0..*consistent_flow_to_move)
                    .collect::<Vec<_>>();
                consistently_moved_supply.retain_mut(|b_t| {
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
}
