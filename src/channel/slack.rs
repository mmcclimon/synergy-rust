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
    fn start(&mut self, tx: mpsc::Sender<event::Event>) {
        info!("starting slack channel");

        let (tx, rx) = mpsc::channel();
        let mut client = client::new();

        let handle = thread::spawn(move || {
            client.listen(tx);
        });

        for raw_event in rx {
            debug!("got raw event: {:?}", raw_event);
        }

        handle.join().unwrap();
    }
}
