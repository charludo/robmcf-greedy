use log::LevelFilter;
use std::io::Write;
use text_io::read;

use robmcf_greedy::Network;
use std::time::Instant;

fn main() {
    setup_logger();

    // let mut n = Network::from_file("error.json");
    let mut n = Network::from_random(
        20,       // num_vertices,
        0.6,      // connectedness,
        0.3,      // supply_density,
        2,        // num_scenarios,
        (3, 8),   // range_supply,
        (15, 40), // range_capacity,
        (4, 8),   // range_cost,
        5,        // num_fixed_arcs,
    );

    n.validate_network();
    println!("{}", n);

    let start_preprocess = Instant::now();
    n.preprocess();
    let elapsed_preprocess = start_preprocess.elapsed();

    let start_solve = Instant::now();
    n.solve();
    let elapsed_solve = start_solve.elapsed();

    println!("{}", n);
    n.validate_solution();

    println!(
        "Creating the network took {}s and {}ms.",
        elapsed_preprocess.as_secs(),
        elapsed_preprocess.subsec_millis()
    );
    println!(
        "Finding a greedy solution took {}s and {}ms.",
        elapsed_solve.as_secs(),
        elapsed_solve.subsec_millis()
    );

    print!("Enter filename or leave blank to skip serializing: ");
    let filename: String = read!("{}\n");
    if !filename.is_empty() {
        n.serialize(&filename);
    }
}

fn setup_logger() {
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{style}{}{style:#} {}:{}] - {} ",
                record.level(),
                match record.file() {
                    Some(r) => r,
                    None => "",
                },
                match record.line() {
                    Some(r) => r.to_string(),
                    None => "".to_string(),
                },
                record.args()
            )
        })
        .init();

    log::debug!("Set up logging.");
}
