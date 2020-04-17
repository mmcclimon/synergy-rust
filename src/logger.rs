extern crate env_logger;

use env_logger::fmt::{Color, Style, StyledValue};
use log::Level;

// This is mostly an excuse to play with the logging stuff. Our log will be
// configured with SYNERGY_LOG, and we'll filter out everything else by default.
// Maybe later, we'll want to support RUST_LOG as well to handle logging from
// other modules.
pub fn init() {
    let mut logger = env_logger::Builder::new();

    // synergy_log will only apply to our module
    if let Ok(ref level) = std::env::var("SYNERGY_LOG") {
        let level = if level == "" { "info" } else { level };
        logger.parse_filters(&format!("synergy_rust={}", level));
    }

    logger.format(|f, record| {
        use std::io::Write;

        let mut style = f.style();
        let level = colored_level(&mut style, record.level());

        writeln!(f, "{} [{}] {}", level, f.timestamp_millis(), record.args())
    });
    logger.init();
}

// stolen from pretty_env_logger
fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE:"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG:"),
        Level::Info => style.set_color(Color::Green).value("INFO: "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN: "),
        Level::Error => style.set_color(Color::Red).value("ERROR:"),
    }
}
