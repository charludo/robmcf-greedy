use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum CostFunction {
    Max,
    Average,
    Median,
}

impl CostFunction {
    pub fn apply(&self, costs: Vec<usize>) -> usize {
        match self {
            CostFunction::Max => *costs.iter().max().unwrap_or(&usize::MAX),
            CostFunction::Average => {
                ((costs.iter().sum::<usize>() as f32) / (costs.len() as f32)) as usize
            }
            CostFunction::Median => {
                costs.clone().sort();
                costs[costs.len() / 2]
            }
        }
    }
}
