use std::cmp::max;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum RelativeDrawFunction {
    Linear,
    LinearWithSlack,
    LinearNonNeg,
    LinearNonNegWithSlack,

    Quadratic,
    QuadraticWithSlack,
    QuadraticNonNeg,
    QuadraticNonNegWithSlack,

    Cubic,
    CubicWithSlack,
    CubicNonNeg,
    CubicNonNegWithSlack,

    Exponential,
    ExponentialWithSlack,
    ExponentialNonNeg,
    ExponentialNonNegWithSlack,
}

impl RelativeDrawFunction {
    pub fn apply(&self, global: i64, scenario: i64, slack: usize) -> i64 {
        match self {
            RelativeDrawFunction::Linear => Self::to_power(global, scenario, 0, 1),
            RelativeDrawFunction::LinearWithSlack => Self::to_power(global, scenario, slack, 1),
            RelativeDrawFunction::LinearNonNeg => max(0, Self::to_power(global, scenario, 0, 1)),
            RelativeDrawFunction::LinearNonNegWithSlack => {
                max(0, Self::to_power(global, scenario, slack, 1))
            }

            RelativeDrawFunction::Quadratic => Self::to_power(global, scenario, 0, 2),
            RelativeDrawFunction::QuadraticWithSlack => Self::to_power(global, scenario, slack, 2),
            RelativeDrawFunction::QuadraticNonNeg => max(0, Self::to_power(global, scenario, 0, 2)),
            RelativeDrawFunction::QuadraticNonNegWithSlack => {
                max(0, Self::to_power(global, scenario, slack, 2))
            }

            RelativeDrawFunction::Cubic => Self::to_power(global, scenario, 0, 3),
            RelativeDrawFunction::CubicWithSlack => Self::to_power(global, scenario, slack, 3),
            RelativeDrawFunction::CubicNonNeg => max(0, Self::to_power(global, scenario, 0, 3)),
            RelativeDrawFunction::CubicNonNegWithSlack => {
                max(0, Self::to_power(global, scenario, slack, 3))
            }

            RelativeDrawFunction::Exponential => Self::exponential(global, scenario, 0),
            RelativeDrawFunction::ExponentialWithSlack => {
                Self::exponential(global, scenario, slack)
            }
            RelativeDrawFunction::ExponentialNonNeg => {
                max(0, Self::exponential(global, scenario, 0))
            }
            RelativeDrawFunction::ExponentialNonNegWithSlack => {
                max(0, Self::exponential(global, scenario, slack))
            }
        }
    }

    fn to_power(global: i64, scenario: i64, slack: usize, e: u32) -> i64 {
        let difference = max(0, global - slack as i64) - scenario;
        let draw = (max(0, global - slack as i64) - scenario).pow(e);

        // Allow negative draw for even exponents
        if difference < 0 && e % 2 == 1 {
            -draw
        } else {
            draw
        }
    }

    fn exponential(global: i64, scenario: i64, slack: usize) -> i64 {
        let difference = max(0, global - slack as i64) - scenario;

        if difference < 0 {
            -(difference.abs() as f64).exp() as i64
        } else {
            (difference as f64).exp() as i64
        }
    }
}
