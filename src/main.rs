#[macro_use]
extern crate log;

mod channel;
mod event;
mod hub;
mod logger;

fn main() {
    logger::init();

    let hub = hub::new("synergy");
    hub.run();
}
