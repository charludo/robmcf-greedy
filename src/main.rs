mod algorithms;
mod matrix;
mod network;
mod util;
use std::time::Instant;

use algorithms::greedy;
use network::Network;
use util::setup_logger;

fn main() {
    setup_logger();

    let start_creation = Instant::now();
    let mut n = Network::from_file("network.json");
    let elapsed_creation = start_creation.elapsed();

    let start_greedy = Instant::now();
    greedy(n.auxiliary_network.as_mut().unwrap());
    let elapsed_greedy = start_greedy.elapsed();

    println!("{}", n);
    println!(
        "Creating the network took {}s and {}ms.",
        elapsed_creation.as_secs(),
        elapsed_creation.subsec_millis()
    );
    println!(
        "Finding a greedy solution took {}s and {}ms.",
        elapsed_greedy.as_secs(),
        elapsed_greedy.subsec_millis()
    );
}
