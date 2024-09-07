use log::LevelFilter;
use std::io::Write;

pub(crate) fn setup_logger(level: LevelFilter) {
    env_logger::builder()
        .filter(None, level)
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "[{style}{}{style:#} {}:{}] - {} ",
                record.level(),
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                record.args()
            )
        })
        .init();

    log::debug!("Set up logging.");
}
