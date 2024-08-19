use std::{collections::HashMap, fmt::Display};

use crate::matrix::Matrix;

use super::b_tuple::BTuple;

#[derive(Debug)]
pub(crate) struct Scenario {
    pub(crate) capacities: Matrix<usize>,
    pub(crate) b_tuples_free: Vec<Box<BTuple>>,
    pub(crate) b_tuples_fixed: HashMap<(usize, usize), Vec<Box<BTuple>>>,
    pub(crate) successor_map: Matrix<usize>,
    pub(crate) distance_map: Matrix<usize>,
}

impl Scenario {
    pub(crate) fn waiting_at(&self, fixed_arc: &(usize, usize)) -> usize {
        match self.b_tuples_fixed.get(&fixed_arc) {
            Some(vec) => vec.len(),
            None => 0,
        }
    }

    pub(crate) fn closest_fixed_arc(&self, fixed_arcs: &Vec<(usize, usize)>) -> (usize, usize) {
        let mut dist_to_closest_fixed_arc = usize::MAX;
        let mut fixed_arc: (usize, usize) = fixed_arcs[0];

        fixed_arcs.iter().for_each(|f_a| {
            let dist_to_f_a = self.distance_map.get(f_a.0, f_a.1);
            if *dist_to_f_a < dist_to_closest_fixed_arc {
                dist_to_closest_fixed_arc = *dist_to_f_a;
                fixed_arc = *f_a;
            }
        });

        fixed_arc
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
