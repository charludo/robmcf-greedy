use std::{collections::HashMap, fmt::Display};

use super::b_tuple::BTuple;

#[derive(Debug)]
pub(crate) struct Scenario {
    pub(crate) id: usize,
    pub(crate) b_tuples_free: Vec<Box<BTuple>>,
    pub(crate) b_tuples_fixed: HashMap<usize, Vec<Box<BTuple>>>,
}

impl Scenario {
    pub(crate) fn waiting_at(&self, fixed_arc: usize) -> usize {
        self.b_tuples_fixed.get(&fixed_arc).unwrap_or(&vec![]).len()
    }

    pub(crate) fn get_relative_draws(
        &self,
        global_waiting: &HashMap<usize, usize>,
    ) -> HashMap<usize, i32> {
        let mut relative_draws = HashMap::new();
        for &key in self.b_tuples_fixed.keys().chain(global_waiting.keys()) {
            let scenario_draw = self.waiting_at(key);
            let global_draw = *global_waiting.get(&key).unwrap_or(&0);
            let relative_draw = (global_draw as i32) - (scenario_draw as i32);
            relative_draws.insert(key, relative_draw);
        }
        relative_draws
    }
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr: Vec<String> = vec![];
        str_repr.push(format!("{} free supply", self.b_tuples_free.len()));
        self.b_tuples_fixed.iter().for_each(|(k, v)| {
            str_repr.push(format!("{} supply waiting at {}", v.len(), k));
        });
        write!(f, "Scenario {}: ( {} )", self.id, str_repr.join(", "))
    }
}
