use crate::{
    network::{Network, Vertex},
    Matrix, Options,
};
use rand::Rng;

#[allow(clippy::too_many_arguments)]
impl Network {
    pub fn from_random(
        options: &Options,
        num_vertices: usize,
        arc_density: f64,
        supply_density: f64,
        num_scenarios: usize,
        umin: usize,
        umax: usize,
        cmin: usize,
        cmax: usize,
        bmin: usize,
        bmax: usize,
        num_fixed_arcs: usize,
        consecutive_fixed_arcs: bool,
    ) -> Self {
        let mut network = Network {
            vertices: vec![],
            capacities: Matrix::empty(),
            costs: Matrix::empty(),
            balances: vec![],
            fixed_arcs: vec![],
            auxiliary_network: None,
            baseline: None,
            solutions: None,
            options: options.clone(),
        };

        network.randomize_vertices(num_vertices);
        network.randomize_capacities(arc_density, umin, umax);
        network.randomize_costs(cmin, cmax);
        network.randomize_scenarios(num_scenarios, supply_density, bmin, bmax);
        network.randomize_fixed_arcs(num_fixed_arcs, consecutive_fixed_arcs);

        network
    }

    pub fn randomize_vertices(&mut self, num_vertices: usize) {
        log::debug!("Randomizing vertices: num_vertices={num_vertices}");
        let mut rng = rand::thread_rng();
        self.vertices = (1..=num_vertices)
            .map(|i| Vertex {
                name: format!("v{}", i),
                x: rng.gen_range((-100 * num_vertices as i64)..(100 * num_vertices as i64)) as f32,
                y: rng.gen_range((-100 * num_vertices as i64)..(100 * num_vertices as i64)) as f32,
            })
            .collect();
    }

    pub fn randomize_capacities(&mut self, arc_density: f64, umin: usize, umax: usize) {
        log::debug!("Randomizing capacities: arc_density={arc_density}, umin={umin}, umax={umax}.");
        self.capacities =
            self.generate_random_matrix(self.vertices.len(), arc_density, (umin, umax));
    }

    pub fn randomize_costs(&mut self, cmin: usize, cmax: usize) {
        log::debug!("Randomizing costs: cmin={cmin}, cmax={cmax}.");
        self.costs = self.generate_random_matrix(self.vertices.len(), 1.0, (cmin, cmax));
    }

    pub fn randomize_scenarios(
        &mut self,
        num_scenarios: usize,
        supply_density: f64,
        bmin: usize,
        bmax: usize,
    ) {
        log::debug!("Randomizing scenarios: num_scenarios={num_scenarios}, supply_density={supply_density}, bmin={bmin}, bmax={bmax}.");
        self.balances = (0..num_scenarios)
            .map(|_| self.generate_random_matrix(self.vertices.len(), supply_density, (bmin, bmax)))
            .collect();
    }

    pub fn randomize_fixed_arcs(&mut self, num_fixed_arcs: usize, consecutive: bool) {
        log::debug!(
            "Randomizing fixed arcs: num_fixed_arcs={num_fixed_arcs}, consecutive={consecutive}."
        );
        let mut fixed_arcs: Vec<(usize, usize)> = Vec::new();
        let mut previous = usize::MAX;
        for _ in 0..num_fixed_arcs {
            let mut rng = rand::thread_rng();
            let a0 = if !consecutive || previous == usize::MAX {
                rng.gen_range(0..self.vertices.len())
            } else {
                previous
            };
            let mut a1 = rng.gen_range(0..self.vertices.len());
            while a0 == a1 {
                a1 = rng.gen_range(0..self.vertices.len());
            }
            fixed_arcs.push((a0, a1));
            previous = a1;
        }
        self.fixed_arcs = fixed_arcs;
    }

    fn generate_random_vec(
        &self,
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
        &self,
        num_vertices: usize,
        connectedness: f64,
        range_values: (usize, usize),
    ) -> Matrix<usize> {
        let mut matrix = Matrix::from_rows(
            &(0..num_vertices)
                .map(|_| self.generate_random_vec(num_vertices, connectedness, range_values))
                .collect::<Vec<Vec<usize>>>(),
        );
        for v in 0..num_vertices {
            matrix.set(v, v, 0);
        }
        matrix
    }
}
