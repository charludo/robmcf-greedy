mod algorithms;
mod matrix;
mod network;
mod util;
use network::Network;
use util::setup_logger;

fn main() {
    setup_logger();

    let n = Network::from_file("network.json");
    println!("{}", n);
}
