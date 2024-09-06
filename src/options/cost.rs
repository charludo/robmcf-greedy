use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab-case")]
pub enum CostFunction {
    Max,
    Mean,
    Median,
}

impl CostFunction {
    pub fn apply(&self, costs: &Vec<usize>) -> usize {
        match self {
            CostFunction::Max => *costs.iter().max().unwrap_or(&usize::MAX),
            CostFunction::Mean => {
                ((costs.iter().sum::<usize>() as f32) / (costs.len() as f32)) as usize
            }
            CostFunction::Median => {
                let mut costs = costs.clone();
                costs.sort();
                (costs[costs.len() / 2] + costs[(costs.len() - 1) / 2]) / 2
            }
        }
    }
}
