use std::sync::Arc;

use crate::matrix::Matrix;

#[derive(Debug, Clone)]
pub(crate) struct BTuple {
    pub(crate) s: usize,
    pub(crate) t: usize,
    pub(crate) intermediate_arc_set: Arc<Matrix<bool>>,
}

impl std::fmt::Display for BTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "(s={}, t={}, A(s,t)={:?})",
            self.s,
            self.t,
            self.intermediate_arc_set
                .indices()
                .filter(|(s, t)| *self.intermediate_arc_set.get(*s, *t))
                .map(|(s, t)| (s, t))
                .collect::<Vec<(usize, usize)>>(),
        )
    }
}
