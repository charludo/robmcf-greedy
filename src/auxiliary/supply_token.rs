use crate::matrix::Matrix;

#[derive(Debug, Clone)]
pub(crate) struct SupplyToken {
    pub(crate) origin: usize,
    pub(crate) s: usize,
    pub(crate) t: usize,

    pub(crate) intermediate_arc_set: Matrix<bool>,
    pub(crate) distances: Matrix<usize>,
    pub(crate) successors: Matrix<usize>,
}

impl SupplyToken {
    pub(crate) fn needs_refresh(&self, capacities: &Matrix<usize>) -> bool {
        self.intermediate_arc_set
            .indices()
            .filter(|&(i, j)| *self.intermediate_arc_set.get(i, j))
            .any(|(i, j)| *capacities.get(i, j) == 0)
    }
}

impl std::fmt::Display for SupplyToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(origin={}, destination={}, currently at={})",
            self.origin, self.t, self.s,
        )
    }
}
