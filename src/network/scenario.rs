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
        match self.b_tuples_fixed.get(&fixed_arc) {
            Some(vec) => vec.len(),
            None => 0,
        }
    }

    pub(crate) fn waiting(&self, fixed_arcs: &Vec<usize>) -> HashMap<usize, usize> {
        let mut wait_map: HashMap<usize, usize> = HashMap::new();
        fixed_arcs.iter().for_each(|fixed_arc| {
            wait_map.insert(*fixed_arc, self.waiting_at(*fixed_arc));
        });
        wait_map
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
