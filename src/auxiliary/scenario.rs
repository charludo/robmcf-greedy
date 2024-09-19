use std::{collections::HashMap, fmt::Display};

use crate::{options::RelativeDrawFunction, Matrix};

use super::{network_state::NetworkState, supply_token::SupplyToken};

#[derive(Debug, Clone)]
pub(crate) struct Scenario {
    pub(crate) id: usize,
    pub(crate) supply_tokens: Vec<SupplyToken>,
    pub(crate) slack: usize,
    pub(crate) slack_used: usize,
    pub(crate) supply_remaining: Matrix<usize>,
    pub(crate) network_state: NetworkState,
}

impl Scenario {
    pub(crate) fn refresh_relative_draws(
        &mut self,
        fixed_arc_loads: &HashMap<(usize, usize), Vec<i64>>,
        draw_fn: &RelativeDrawFunction,
    ) {
        for &(a_0, a_1) in fixed_arc_loads.keys() {
            let peer_usage = fixed_arc_loads.get(&(a_0, a_1)).unwrap();
            let local_usage = *self.network_state.arc_loads.get(a_0, a_1);
            let relative_draw =
                draw_fn.apply(peer_usage, local_usage as i64, self.slack - self.slack_used);
            self.network_state
                .relative_draws
                .insert((a_0, a_1), relative_draw);
        }
    }
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scenario {}: {} supply tokens",
            self.id,
            self.supply_tokens.len()
        )
    }
}
