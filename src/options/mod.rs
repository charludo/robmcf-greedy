mod cost;
mod delta;
mod relative_draw;
mod slack;
mod solve_method;

pub struct Options {
    pub cost_fn: cost::CostFunction,
    pub delta_fn: delta::DeltaFunction,
    pub relative_draw_fn: relative_draw::RelativeDrawFunction,
    pub slack_fn: slack::SlackFunction,
    pub remainder_solve_method: solve_method::RemainderSolveMethod,
}
