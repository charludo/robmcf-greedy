use log::LevelFilter;
use std::io::Write;

pub mod random_network;

pub(crate) fn setup_logger() {
    env_logger::builder()
        .filter(None, LevelFilter::Error)
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
