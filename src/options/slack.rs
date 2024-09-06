use crate::matrix::Matrix;

pub enum SlackFunction {
    BalanceMin,
    DifferenceToMax,
    DifferenceToMaxPlusBalanceMin,
}

impl SlackFunction {
    pub fn apply(&self, balances: &Vec<Matrix<usize>>) -> Vec<usize> {
        match self {
            SlackFunction::BalanceMin => vec![Self::min_balance(balances); balances.len()],
            SlackFunction::DifferenceToMax => Self::differences(balances, 0),
            SlackFunction::DifferenceToMaxPlusBalanceMin => {
                Self::differences(balances, Self::min_balance(balances))
            }
        }
    }

    fn min_balance(balances: &Vec<Matrix<usize>>) -> usize {
        balances.iter().map(|b| b.sum()).min().unwrap_or(0)
    }

    fn differences(balances: &Vec<Matrix<usize>>, offset: usize) -> Vec<usize> {
        let max = balances.iter().map(|b| b.sum()).max().unwrap_or(0);
        balances.iter().map(|b| max - b.sum() + offset).collect()
    }
}
