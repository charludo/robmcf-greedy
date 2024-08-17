use std::{collections::HashMap, fmt::Display};

use crate::matrix::Matrix;

use super::b_tuple::BTuple;

#[derive(Debug)]
pub(super) struct Scenario {
    pub(super) capacities: Matrix<usize>,
    pub(super) b_tuples_free: Vec<Box<BTuple>>,
    pub(super) b_tuples_fixed: HashMap<(usize, usize), Vec<Box<BTuple>>>,
    pub(super) successor_map: Matrix<usize>,
    pub(super) distance_map: Matrix<usize>,
}

impl Scenario {
    pub(super) fn waiting_at(&self, fixed_arc: (usize, usize)) -> usize {
        match self.b_tuples_fixed.get(&fixed_arc) {
            Some(vec) => vec.len(),
            None => 0,
        }
    }
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr: Vec<String> = vec![];
        str_repr.push("(".to_string());
        str_repr.push(format!("{:>6} free supply", self.b_tuples_free.len()));
        self.b_tuples_fixed.iter().for_each(|(k, v)| {
            str_repr.push(format!(
                "{:>6} supply waiting at ({}, {})",
                v.len(),
                k.0,
                k.1
            ));
        });
        str_repr.push(")".to_string());
        write!(f, "{}", str_repr.join("\n"))
    }
}
