use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum DeltaFunction {
    LinearLow,
    LinearMedium,
    LinearHigh,

    LogarithmicLow,
    LogarithmicMedium,
    LogarithmicHigh,
}

impl DeltaFunction {
    pub fn apply(&self, x: usize) -> usize {
        match self {
            DeltaFunction::LinearLow => (1.5 * x as f32).floor() as usize,
            DeltaFunction::LinearMedium => (2. * x as f32).floor() as usize,
            DeltaFunction::LinearHigh => (3. * x as f32).floor() as usize,

            DeltaFunction::LogarithmicLow => Self::logarithmic(x, 5.),
            DeltaFunction::LogarithmicMedium => Self::logarithmic(x, 10.),
            DeltaFunction::LogarithmicHigh => Self::logarithmic(x, 20.),
        }
    }

    fn logarithmic(x: usize, k: f32) -> usize {
        x + (k * (x as f32 + 1.).ln()).floor() as usize
    }
}
