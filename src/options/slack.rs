use crate::matrix::Matrix;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum SlackFunction {
    BalanceMin,
    DifferenceToMax,
    DifferenceToMaxPlusMin,
}

impl SlackFunction {
    pub fn apply(&self, balances: &[Matrix<usize>]) -> Vec<usize> {
        match self {
            SlackFunction::BalanceMin => vec![Self::min_balance(balances); balances.len()],
            SlackFunction::DifferenceToMax => Self::differences(balances, 0),
            SlackFunction::DifferenceToMaxPlusMin => {
                Self::differences(balances, Self::min_balance(balances))
            }
        }
    }

    fn min_balance(balances: &[Matrix<usize>]) -> usize {
        balances.iter().map(|b| b.sum()).min().unwrap_or(0)
    }

    fn differences(balances: &[Matrix<usize>], offset: usize) -> Vec<usize> {
        let max = balances.iter().map(|b| b.sum()).max().unwrap_or(0);
        balances.iter().map(|b| max - b.sum() + offset).collect()
    }
}
