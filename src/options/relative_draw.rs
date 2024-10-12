use std::cmp::max;
use strum::Display;

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone, Display)]
#[clap(rename_all = "kebab-case")]
pub enum RelativeDrawFunction {
    None,

    Linear,
    LinearNonNeg,

    Quadratic,
    QuadraticNonNeg,

    Cubic,
    CubicNonNeg,

    Exponential,
    ExponentialNonNeg,

    PeerPressure,
}

impl RelativeDrawFunction {
    pub fn apply(&self, peers: &[i64], scenario: i64) -> i64 {
        match self {
            RelativeDrawFunction::None => 0,
            RelativeDrawFunction::Linear => Self::to_power(peers, scenario, 1),
            RelativeDrawFunction::LinearNonNeg => max(0, Self::to_power(peers, scenario, 1)),

            RelativeDrawFunction::Quadratic => Self::to_power(peers, scenario, 2),
            RelativeDrawFunction::QuadraticNonNeg => max(0, Self::to_power(peers, scenario, 2)),

            RelativeDrawFunction::Cubic => Self::to_power(peers, scenario, 3),
            RelativeDrawFunction::CubicNonNeg => max(0, Self::to_power(peers, scenario, 3)),

            RelativeDrawFunction::Exponential => Self::exponential(peers, scenario),
            RelativeDrawFunction::ExponentialNonNeg => max(0, Self::exponential(peers, scenario)),

            RelativeDrawFunction::PeerPressure => Self::peer_pressure(peers, scenario),
        }
    }

    fn to_power(peers: &[i64], scenario: i64, e: u32) -> i64 {
        let waiting_at_peers = peers.iter().sum::<i64>();
        let difference = waiting_at_peers - peers.len() as i64 * scenario;
        let draw = difference.pow(e);

        // Allow negative draw for even exponents
        if difference < 0 && e % 2 == 0 {
            -draw
        } else {
            draw
        }
    }

    fn exponential(peers: &[i64], scenario: i64) -> i64 {
        let difference = peers.iter().sum::<i64>() - peers.len() as i64 * scenario;

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
