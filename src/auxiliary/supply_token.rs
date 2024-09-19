#[derive(Debug, Clone)]
pub(crate) struct SupplyToken {
    pub(crate) origin: usize,
    pub(crate) s: usize,
    pub(crate) t: usize,
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
