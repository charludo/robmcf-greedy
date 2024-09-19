use std::{collections::HashMap, fmt::Display};

use crate::{options::RelativeDrawFunction, Matrix, Result, SolverError};

use super::{network_state::NetworkState, supply_token::SupplyToken};

#[derive(Debug, Clone)]
pub(crate) struct Scenario {
    pub(crate) id: usize,
    pub(crate) tokens_free: Vec<SupplyToken>,
    pub(crate) tokens_fixed: HashMap<usize, Vec<SupplyToken>>,
    pub(crate) slack: usize,
    pub(crate) slack_used: usize,
    pub(crate) supply_remaining: Matrix<usize>,
    pub(crate) network_state: NetworkState,
}

impl Scenario {
    pub(crate) fn waiting_at(&self, fixed_arc: usize) -> usize {
        self.tokens_fixed.get(&fixed_arc).unwrap_or(&vec![]).len()
    }

    pub(crate) fn refresh_relative_draws(
        &mut self,
        global_waiting: &HashMap<usize, usize>,
        draw_fn: &RelativeDrawFunction,
    ) {
        for &key in self.tokens_fixed.keys().chain(global_waiting.keys()) {
            let scenario_draw = self.waiting_at(key);
            let global_draw = *global_waiting.get(&key).unwrap_or(&0);
            let relative_draw = draw_fn.apply(
                global_draw as i64,
                scenario_draw as i64,
                self.slack - self.slack_used,
            );
            self.network_state.relative_draws.insert(key, relative_draw);
        }
    }

    pub(crate) fn use_slack(&mut self, amount: usize) -> Result<()> {
        if self.slack_used + amount > self.slack {
            Err(SolverError::NoSlackLeftError(self.id))
        } else {
            self.slack_used += amount;
            Ok(())
        }
    }
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr: Vec<String> = vec![];
        str_repr.push(format!("{} free supply", self.tokens_free.len()));
        self.tokens_fixed.iter().for_each(|(k, v)| {
            str_repr.push(format!("{} supply waiting at {}", v.len(), k));
        });
        write!(f, "Scenario {}: ( {} )", self.id, str_repr.join(", "))
    }
}
