extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod channel;
mod event;
mod hub;

fn main() {
    pretty_env_logger::init();

    let hub = hub::new("synergy");
    hub.run();
}
