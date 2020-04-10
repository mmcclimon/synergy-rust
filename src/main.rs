mod synergy;

use synergy::hub;

fn main() {
    let hub = hub::new("synergy");
    hub.run()
}
