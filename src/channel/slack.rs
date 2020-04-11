mod client;

use std::sync::mpsc;
use std::thread;

use crate::channel;
use crate::event;

pub struct Slack {
    name: String,
    api_token: String,
}

pub fn new(name: String, cfg: &channel::ChannelConfig) -> Box<Slack> {
    let api_token = &cfg.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    let channel = Slack {
        name: name,
        api_token: api_token.to_string(),
    };

    return Box::new(channel);
}

impl crate::channel::Channel for Slack {
    fn start(&self, events_channel: mpsc::Sender<event::Event>) -> thread::JoinHandle<()> {
        info!("starting slack channel {}", self.name);

        // we take a copy here to avoid moving &self into the thread.
        let cloned_token = self.api_token.clone();

        return std::thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let client = client::new(cloned_token);

            thread::spawn(move || client.listen(tx));

            for raw_event in rx {
                debug!("got raw event: {:?}", raw_event);
                let e = event::Event {};
                events_channel.send(e).unwrap();
            }
        });
    }
}
