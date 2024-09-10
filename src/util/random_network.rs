use crate::{
    matrix::Matrix,
    network::{Network, Vertex},
    Options,
};
use rand::Rng;

#[allow(clippy::too_many_arguments)]
impl Network {
    pub fn from_random(
        options: &Options,
        num_vertices: usize,
        connectedness: f64,
        supply_density: f64,
        num_scenarios: usize,
        range_supply: (usize, usize),
        range_capacity: (usize, usize),
        range_cost: (usize, usize),
        num_fixed_arcs: usize,
    ) -> Self {
        let vertices: Vec<Vertex> = (1..=num_vertices)
            .map(|i| Vertex {
                name: format!("v{}", i),
                x: generate_random_coordinate(num_vertices),
                y: generate_random_coordinate(num_vertices),
            })
            .collect();
        let capacities: Matrix<usize> =
            generate_random_matrix(num_vertices, connectedness, range_capacity);
        let costs: Matrix<usize> = generate_random_matrix(num_vertices, 1.0, range_cost);
        let balances: Vec<Matrix<usize>> = (0..num_scenarios)
            .map(|_| generate_random_matrix(num_vertices, supply_density, range_supply))
            .collect();
        let fixed_arcs: Vec<(usize, usize)> = (0..num_fixed_arcs)
            .map(|_| generate_random_fixed_arc(num_vertices, &capacities))
            .collect();

        Network {
            vertices,
            capacities,
            costs,
            balances,
            fixed_arcs,
            auxiliary_network: None,
            solutions: None,
            options: options.clone(),
        }
    }
}

fn generate_random_coordinate(num_vertices: usize) -> f32 {
    let mut rng = rand::thread_rng();
    let p = rng.gen_range((-100 * num_vertices as i32)..(100 * num_vertices as i32));
    p as f32
}

fn generate_random_fixed_arc(num_vertices: usize, capacities: &Matrix<usize>) -> (usize, usize) {
    let mut rng = rand::thread_rng();

    let mut a0 = rng.gen_range(0..num_vertices);
    let mut a1 = rng.gen_range(0..num_vertices);
    while a0 == a1 || *capacities.get(a0, a1) == 0 {
        a0 = rng.gen_range(0..num_vertices);
        a1 = rng.gen_range(0..num_vertices);
    }

    (a0, a1)
}

fn generate_random_vec(
    num_vertices: usize,
    connectedness: f64,
    range_values: (usize, usize),
) -> Vec<usize> {
    let mut rng = rand::thread_rng();

    (0..num_vertices)
        .map(|_| {
            if rng.gen_bool(connectedness) {
                rng.gen_range(range_values.0..=range_values.1)
            } else {
                0
            }
        })
        .collect()
}

fn generate_random_matrix(
    num_vertices: usize,
    connectedness: f64,
    range_values: (usize, usize),
) -> Matrix<usize> {
    let mut matrix = Matrix::from_rows(
        &(0..num_vertices)
            .map(|_| generate_random_vec(num_vertices, connectedness, range_values))
            .collect::<Vec<Vec<usize>>>(),
    );
    for v in 0..num_vertices {
        matrix.set(v, v, 0);
    }
    matrix
}
