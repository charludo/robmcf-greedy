use std::time::{Duration, Instant};

use robmcf_greedy::Network;

pub(crate) fn run_benchmark(network: &Network, iterations: usize) {
    let (mut preprocess, mut solve): (Duration, Duration) = (Duration::ZERO, Duration::ZERO);

    for _ in 0..iterations {
        let mut n: Network = network.clone();

        let start_preprocess = Instant::now();
        crate::attempt!(n.preprocess());
        let elapsed_preprocess = start_preprocess.elapsed();

        let start_solve = Instant::now();
        crate::attempt!(n.solve());
        let elapsed_solve = start_solve.elapsed();

        preprocess += elapsed_preprocess;
        solve += elapsed_solve;
    }

    preprocess /= iterations as u32;
    solve /= iterations as u32;

    println!(
        "Creating the network took {}s and {}ms on average (n={}).",
        preprocess.as_secs(),
        preprocess.subsec_millis(),
        iterations,
    );
    println!(
        "Finding a greedy solution took {}s and {}ms on average (n={}).",
        solve.as_secs(),
        solve.subsec_millis(),
        iterations,
    );
}
