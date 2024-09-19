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

    PeerPressure,
}

impl RelativeDrawFunction {
    pub fn apply(&self, peers: &[i64], scenario: i64, slack: usize) -> i64 {
        match self {
            RelativeDrawFunction::Linear => Self::to_power(peers, scenario, 0, 1),
            RelativeDrawFunction::LinearWithSlack => Self::to_power(peers, scenario, slack, 1),
            RelativeDrawFunction::LinearNonNeg => max(0, Self::to_power(peers, scenario, 0, 1)),
            RelativeDrawFunction::LinearNonNegWithSlack => {
                max(0, Self::to_power(peers, scenario, slack, 1))
            }

            RelativeDrawFunction::Quadratic => Self::to_power(peers, scenario, 0, 2),
            RelativeDrawFunction::QuadraticWithSlack => Self::to_power(peers, scenario, slack, 2),
            RelativeDrawFunction::QuadraticNonNeg => max(0, Self::to_power(peers, scenario, 0, 2)),
            RelativeDrawFunction::QuadraticNonNegWithSlack => {
                max(0, Self::to_power(peers, scenario, slack, 2))
            }

            RelativeDrawFunction::Cubic => Self::to_power(peers, scenario, 0, 3),
            RelativeDrawFunction::CubicWithSlack => Self::to_power(peers, scenario, slack, 3),
            RelativeDrawFunction::CubicNonNeg => max(0, Self::to_power(peers, scenario, 0, 3)),
            RelativeDrawFunction::CubicNonNegWithSlack => {
                max(0, Self::to_power(peers, scenario, slack, 3))
            }

            RelativeDrawFunction::Exponential => Self::exponential(peers, scenario, 0),
            RelativeDrawFunction::ExponentialWithSlack => Self::exponential(peers, scenario, slack),
            RelativeDrawFunction::ExponentialNonNeg => {
                max(0, Self::exponential(peers, scenario, 0))
            }
            RelativeDrawFunction::ExponentialNonNegWithSlack => {
                max(0, Self::exponential(peers, scenario, slack))
            }

            RelativeDrawFunction::PeerPressure => Self::peer_pressure(peers, scenario),
        }
    }

    fn to_power(peers: &[i64], scenario: i64, slack: usize, e: u32) -> i64 {
        let waiting_at_peers = peers.iter().sum::<i64>();
        let difference = max(0, waiting_at_peers - slack as i64) - peers.len() as i64 * scenario;
        let draw = difference.pow(e);

        // Allow negative draw for even exponents
        if difference < 0 && e % 2 == 0 {
            -draw
        } else {
            draw
        }
    }

    fn exponential(peers: &[i64], scenario: i64, slack: usize) -> i64 {
        let difference =
            max(0, peers.iter().sum::<i64>() - slack as i64) - peers.len() as i64 * scenario;

        // Actually repulse when difference is negative
        if difference < 0 {
            -(difference.abs() as f64).exp() as i64
        } else {
            (difference as f64).exp() as i64
        }
    }

    fn peer_pressure(peers: &[i64], scenario: i64) -> i64 {
        let m = peers.iter().filter(|peer| **peer > scenario).count() as u32;
        peers
            .iter()
            .map(|peer| max(0, peer - scenario))
            .sum::<i64>()
            .pow(m)
    }
}
