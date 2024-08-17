use crate::matrix::Matrix;
use serde::Deserialize;
use serde_json;
use std::{fs::File, io::BufReader};

use super::auxiliary_network::AuxiliaryNetwork;

#[derive(Deserialize, Debug)]
pub struct Network {
    pub vertices: Vec<String>,
    pub capacities: Matrix<usize>,
    pub costs: Matrix<usize>,
    pub balances: Vec<Matrix<usize>>,
    pub fixed_arcs: Vec<(usize, usize)>,
    #[serde(skip)]
    pub arc_loads: Option<Vec<Matrix<usize>>>,
    #[serde(skip)]
    auxiliary_network: Option<Box<AuxiliaryNetwork>>,
}

impl Network {
    pub fn from_file(filename: &str) -> Self {
        let file = match File::open(filename) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to open file \"{}\": {}", filename, msg),
        };
        let reader = BufReader::new(file);

        log::debug!("Deserializing network from {}", filename);
        let mut network: Network = match serde_json::from_reader(reader) {
            Ok(result) => result,
            Err(msg) => panic!("Failed to parse the network: {}", msg),
        };

        network.validate();
        network.preprocess();
        network
    }

    fn validate(&self) {
        let len = self.vertices.len();

        let matrices = [&self.capacities, &self.costs];
        for matrix in matrices {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices u, c have differing dimensions or are not quadratic");
            }
        }

        for matrix in &self.balances {
            if matrix.num_rows() != len || matrix.num_columns() != len {
                panic!("Matrices in b have differing dimensions or are not quadratic");
            }
        }

        log::debug!("Network is valid.");
    }

    fn preprocess(&mut self) {
        log::debug!("Beginning to preprocess network.");
        match self.auxiliary_network {
            Some(_) => {}
            None => self.auxiliary_network = Some(Box::new(AuxiliaryNetwork::from(&*self))),
        }
    }
}
