mod cost;
mod delta;
mod relative_draw;
mod slack;
mod solve_method;

pub use cost::CostFunction;
pub use delta::DeltaFunction;
pub use relative_draw::RelativeDrawFunction;
pub use slack::SlackFunction;
pub use solve_method::RemainderSolveMethod;

#[derive(Debug, Clone)]
pub struct Options {
    pub cost_fn: CostFunction,
    pub delta_fn: DeltaFunction,
    pub relative_draw_fn: RelativeDrawFunction,
    pub slack_fn: SlackFunction,
    pub remainder_solve_method: RemainderSolveMethod,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            cost_fn: CostFunction::Max,
            delta_fn: DeltaFunction::LinearMedium,
            relative_draw_fn: RelativeDrawFunction::Linear,
            slack_fn: SlackFunction::DifferenceToMax,
            remainder_solve_method: RemainderSolveMethod::No,
        }
    }
}
