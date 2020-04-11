#[macro_use]
extern crate log;

mod channel;
mod config;
mod event;
mod hub;
mod logger;

fn main() {
    logger::init();

    let config = config::new("config.toml");

    let hub = hub::new(config);

    hub.run();
}
