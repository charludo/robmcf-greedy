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

impl std::fmt::Display for SupplyToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(origin={}, destination={}, currently at={})",
            self.origin, self.t, self.s,
        )
    }
}
