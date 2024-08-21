use std::{collections::HashMap, fmt::Display};

use crate::{
    algorithms::{floyd_warshall, invert_predecessors},
    matrix::Matrix,
};

use super::b_tuple::BTuple;

#[derive(Debug)]
pub(crate) struct Scenario {
    pub(crate) capacities: Matrix<usize>,
    pub(crate) b_tuples_free: Vec<Box<BTuple>>,
    pub(crate) b_tuples_fixed: HashMap<(usize, usize), Vec<Box<BTuple>>>,
    pub(crate) arc_loads: Matrix<usize>,
    pub(crate) slack: usize,
}

impl Scenario {
    pub(crate) fn waiting_at(&self, fixed_arc: &(usize, usize)) -> usize {
        match self.b_tuples_fixed.get(&fixed_arc) {
            Some(vec) => vec.len(),
            None => 0,
        }
    }

    pub(crate) fn waiting(
        &self,
        fixed_arcs: &Vec<(usize, usize)>,
    ) -> HashMap<(usize, usize), usize> {
        let mut wait_map: HashMap<(usize, usize), usize> = HashMap::new();
        fixed_arcs.iter().for_each(|fixed_arc| {
            wait_map.insert(*fixed_arc, self.waiting_at(fixed_arc));
        });
        wait_map
    }

    pub(crate) fn use_arc(&mut self, s: usize, t: usize, costs: &Matrix<usize>) {
        log::warn!("Scenario has {} free BTuples.", self.b_tuples_free.len());
        let _ = self.arc_loads.increment(s, t);
        let remaining_capacity = self.capacities.decrement(s, t);
        if remaining_capacity == 0 {
            self.refresh_maps(s, t, costs);
        }
    }

    fn refresh_maps_of(&self, b_tuple: &mut BTuple, costs: &Matrix<usize>) {
        log::info!("BTuple {} needs to refresh its maps.", b_tuple);
        let (distance_map, predecessor_map) = floyd_warshall(
            &self
                .capacities
                .apply_mask(&b_tuple.intermediate_arc_set, usize::MAX),
            &costs,
        );
        let successor_map = invert_predecessors(&predecessor_map);

        b_tuple.distance_map = distance_map;
        b_tuple.successor_map = successor_map;
    }

    pub(crate) fn refresh_maps(&mut self, s: usize, t: usize, costs: &Matrix<usize>) {
        log::debug!(
            "Arc ({}->{}) has reached its capacity. Refreshing distance- and successor-maps.",
            s,
            t,
        );

        let mut b_tuples_free = std::mem::take(&mut self.b_tuples_free);
        b_tuples_free
            .iter_mut()
            .filter(|b_tuple| *b_tuple.intermediate_arc_set.get(s, t))
            .for_each(|b_tuple| {
                self.refresh_maps_of(b_tuple, costs);
            });
        self.b_tuples_free = b_tuples_free;

        let mut b_tuples_fixed = std::mem::take(&mut self.b_tuples_fixed);
        b_tuples_fixed.iter_mut().for_each(|(_, b_tuples)| {
            b_tuples
                .iter_mut()
                .filter(|b_tuple| *b_tuple.intermediate_arc_set.get(s, t))
                .for_each(|b_tuple| {
                    self.refresh_maps_of(b_tuple, costs);
                });
        });
        self.b_tuples_fixed = b_tuples_fixed;
    }
}

impl Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr: Vec<String> = vec![];
        str_repr.push(format!("{} free supply", self.b_tuples_free.len()));
        self.b_tuples_fixed.iter().for_each(|(k, v)| {
            str_repr.push(format!("{} supply waiting at ({}, {})", v.len(), k.0, k.1));
        });
        write!(f, "( {} )", str_repr.join(", "))
    }
}
