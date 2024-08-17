use crate::matrix::Matrix;

use super::b_tuple::BTuple;

#[derive(Debug)]
pub(super) struct Scenario {
    pub(super) capacities: Matrix<usize>,
    pub(super) b_tuples_free: Vec<Box<BTuple>>,
    pub(super) b_tuples_fixed: Vec<Vec<Box<BTuple>>>,
    pub(super) successor_map: Matrix<usize>,
    pub(super) distance_map: Matrix<usize>,
}
