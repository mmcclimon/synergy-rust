mod client;

use std::sync::mpsc;

pub struct Slack {
    client: client::Client,
}

pub fn new() -> Box<Slack> {
    debug!("slack::new()");

    let channel = Slack {
        client: client::new(),
    };

    return Box::new(channel);
}

impl super::Channel for Slack {
    fn start(&mut self) {
        info!("starting slack channel");
        self.client.listen();
    }
}
