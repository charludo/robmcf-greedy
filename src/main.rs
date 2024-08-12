use std::{fmt, fs::File, io::BufReader};

use array2d::Array2D;
use serde::{Deserialize, Serialize};

fn main() {
    let n = read_from_file("network.json").unwrap();
    println!("Network: {:?}", n);
    let (dist, prev) = floyd_warshall(&n.u, &n.c);
    println!("Distances: {:?}", dist);
    println!("Predecessors: {:?}", prev);
    println!("Shortest path v1 -> v2: {:?}", shortest_path(&prev, 0, 1));
    println!("Shortest path v1 -> v3: {:?}", shortest_path(&prev, 0, 2));
    println!("Shortest path v2 -> v3: {:?}", shortest_path(&prev, 1, 2));
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkRaw {
    v: Vec<String>,
    u: Vec<Vec<usize>>,
    c: Vec<Vec<usize>>,
    b: Vec<Vec<Vec<usize>>>,
}

#[derive(Debug)]
struct Network {
    v: Vec<String>,
    u: Array2D<usize>,
    c: Array2D<usize>,
    b: Vec<Array2D<usize>>,
}

fn floyd_warshall(
    u: &Array2D<usize>,
    c: &Array2D<usize>,
) -> (Array2D<usize>, Array2D<Option<usize>>) {
    let mut dist: Array2D<usize> = Array2D::filled_with(usize::MAX, c.num_rows(), c.num_columns());
    let mut prev: Array2D<Option<usize>> =
        Array2D::filled_with(None, c.num_rows(), c.num_columns());

    for (x, y) in u
        .indices_row_major()
        .filter(|(x, y)| *u.get(*x, *y).unwrap() > 0)
    {
        // println!("dist {} -> {} is {}", x+1, y+1, c.get(x, y).unwrap());
        let _ = dist.set(x, y, *c.get(x, y).unwrap());
        let _ = prev.set(x, y, Some(x));
    }
    for v in 0..u.num_rows() {
        // println!("pred. of {} is {} with distance 0", v+1, v+1);
        let _ = dist.set(v, v, 0);
        let _ = prev.set(v, v, Some(v));
    }
    for k in 0..u.num_rows() {
        for i in 0..u.num_rows() {
            for j in 0..u.num_rows() {
                if *dist.get(i, k).unwrap() < usize::MAX
                    && *dist.get(k, j).unwrap() < usize::MAX
                    && *dist.get(i, j).unwrap() > dist.get(i, k).unwrap() + dist.get(k, j).unwrap()
                {
                    // println!("new dist {} -> {} is {}", i+1, j+1, dist.get(i, k).unwrap() + dist.get(k, j).unwrap());
                    let _ = dist.set(i, j, dist.get(i, k).unwrap() + dist.get(k, j).unwrap());
                    let _ = prev.set(i, j, *prev.get(k, j).unwrap());
                }
            }
        }
    }

    (dist, prev)
}

fn shortest_path(prev: &Array2D<Option<usize>>, s: usize, mut t: usize) -> Vec<usize> {
    let mut p = match prev.get(s, t).unwrap() {
        Some(_) => vec![t],
        None => return vec![],
    };

    while s != t {
        t = prev.get(s, t).unwrap().expect("");
        p.push(t);
    }

    p.reverse();
    p
}

fn read_from_file(filename: &str) -> Result<Network, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let network_raw: NetworkRaw = serde_json::from_reader(reader)?;

    let network = Network {
        v: network_raw.v,
        u: Array2D::from_rows(&network_raw.u)?,
        c: Array2D::from_rows(&network_raw.c)?,
        b: network_raw
            .b
            .into_iter()
            .map(|b| Array2D::from_rows(&b))
            .collect::<Result<Vec<_>, _>>()?,
    };

    validate_network(&network)?;

    Ok(network)
}

#[derive(Debug)]
struct NetworkShapeError {
    msg: String,
}
impl NetworkShapeError {
    pub fn new(msg: &str) -> NetworkShapeError {
        NetworkShapeError {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for NetworkShapeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for NetworkShapeError {}

impl From<&str> for NetworkShapeError {
    fn from(msg: &str) -> Self {
        NetworkShapeError::new(msg)
    }
}

fn validate_network(n: &Network) -> Result<(), NetworkShapeError> {
    let len = n.v.len();

    let matrices = [&n.u, &n.c];
    for matrix in matrices {
        if matrix.num_rows() != len || matrix.num_columns() != len {
            return Err("Matrices u, c have differing dimensions or are not quadratic")?;
        }
    }

    for matrix in &n.b {
        if matrix.num_rows() != len || matrix.num_columns() != len {
            return Err("Matrices in b have differing dimensions or are not quadratic")?;
        }
    }

    Ok(())
}
