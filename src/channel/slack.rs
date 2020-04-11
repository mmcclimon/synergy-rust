mod client;

use std::sync::mpsc;
use std::thread;

use crate::event;

pub struct Slack {}

pub fn new() -> Box<Slack> {
    debug!("slack::new()");

    let channel = Slack {};

    return Box::new(channel);
}

impl crate::channel::Channel for Slack {
    fn start(&self, events_channel: mpsc::Sender<event::Event>) {
        info!("starting slack channel");

        let (tx, rx) = mpsc::channel();
        let client = client::new();

        let handle = thread::spawn(move || {
            client.listen(tx);
        });

        for raw_event in rx {
            debug!("got raw event: {:?}", raw_event);
            let e = event::Event {};
            events_channel.send(e).unwrap();
        }

        handle.join().unwrap();
    }
}
