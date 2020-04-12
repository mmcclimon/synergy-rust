#[macro_use]
extern crate log;

mod channel;
mod config;
mod environment;
mod event;
mod hub;
mod logger;
mod user_directory;

fn main() {
    logger::init();

    let config = config::new("config.toml");
    let hub = hub::new(config);
    hub.run();
}
