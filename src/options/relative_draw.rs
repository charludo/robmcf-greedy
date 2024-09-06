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
}

impl RelativeDrawFunction {
    pub fn apply(&self, global: i32, scenario: i32, slack: usize) -> i32 {
        match self {
            RelativeDrawFunction::Linear => Self::exponential(global, scenario, 0, 1),
            RelativeDrawFunction::LinearWithSlack => Self::exponential(global, scenario, slack, 1),
            RelativeDrawFunction::LinearNonNeg => max(0, Self::exponential(global, scenario, 0, 1)),
            RelativeDrawFunction::LinearNonNegWithSlack => {
                max(0, Self::exponential(global, scenario, slack, 1))
            }

            RelativeDrawFunction::Quadratic => Self::exponential(global, scenario, 0, 2),
            RelativeDrawFunction::QuadraticWithSlack => {
                Self::exponential(global, scenario, slack, 2)
            }
            RelativeDrawFunction::QuadraticNonNeg => {
                max(0, Self::exponential(global, scenario, 0, 2))
            }
            RelativeDrawFunction::QuadraticNonNegWithSlack => {
                max(0, Self::exponential(global, scenario, slack, 2))
            }

            RelativeDrawFunction::Cubic => Self::exponential(global, scenario, 0, 3),
            RelativeDrawFunction::CubicWithSlack => Self::exponential(global, scenario, slack, 3),
            RelativeDrawFunction::CubicNonNeg => max(0, Self::exponential(global, scenario, 0, 3)),
            RelativeDrawFunction::CubicNonNegWithSlack => {
                max(0, Self::exponential(global, scenario, slack, 3))
            }
        }
    }

    fn exponential(global: i32, scenario: i32, slack: usize, e: u32) -> i32 {
        let difference = max(0, global - slack as i32) - scenario;
        let draw = (max(0, global - slack as i32) - scenario).pow(e);

        // Allow negative draw for even exponents
        if difference < 0 && draw > 0 {
            -draw
        } else {
            draw
        }
    }
}
