use crate::matrix::Matrix;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
#[allow(non_camel_case_types)]
pub enum SlackFunction {
    Const_10,
    Const_100,
    Const_1000,
    Unlimited,

    BalanceMin,
    DifferenceToMax,
    DifferenceToMaxPlusMin,

    DifferenceToMaxPlus_10,
    DifferenceToMaxPlus_100,
    DifferenceToMaxPlus_1000,
}

impl SlackFunction {
    pub fn apply(&self, balances: &[Matrix<usize>]) -> Vec<usize> {
        match self {
            SlackFunction::Const_10 => vec![10; balances.len()],
            SlackFunction::Const_100 => vec![100; balances.len()],
            SlackFunction::Const_1000 => vec![1000; balances.len()],
            SlackFunction::Unlimited => vec![usize::MAX; balances.len()],

            SlackFunction::BalanceMin => vec![Self::min_balance(balances); balances.len()],
            SlackFunction::DifferenceToMax => Self::differences(balances, 0),
            SlackFunction::DifferenceToMaxPlusMin => {
                Self::differences(balances, Self::min_balance(balances))
            }
            SlackFunction::DifferenceToMaxPlus_10 => Self::differences(balances, 10),
            SlackFunction::DifferenceToMaxPlus_100 => Self::differences(balances, 100),
            SlackFunction::DifferenceToMaxPlus_1000 => Self::differences(balances, 1000),
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
