mod channel;
mod hub;

fn main() {
    let hub = hub::new("synergy");
    hub.run()
}
