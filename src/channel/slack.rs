mod client;

use std::sync::mpsc;
use std::thread;

use crate::channel::{Channel, ChannelConfig};
use crate::event::{Event, EventType};

pub struct Slack {
    name: String,
    api_token: String,
}

pub fn new(name: String, cfg: &ChannelConfig) -> Box<Slack> {
    let api_token = &cfg.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    let channel = Slack {
        name: name,
        api_token: api_token.to_string(),
    };

    return Box::new(channel);
}

impl Channel for Slack {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn start(&self, events_channel: mpsc::Sender<Event>) -> thread::JoinHandle<()> {
        info!("starting slack channel {}", self.name);

        // we take a copy here to avoid moving &self into the thread.
        let cloned_token = self.api_token.clone();

        return std::thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let client = client::new(cloned_token);

            thread::spawn(move || client.listen(tx));

            for raw_event in rx {
                let e = Event {
                    kind: EventType::Message,
                    // TODO: fill these in properly
                    from_user: None, // TODO
                    text: raw_event.text,
                    is_public: false,
                    was_targeted: true,
                    from_address: raw_event.user,
                    conversation_address: raw_event.channel,
                    from_channel: String::from("slack"),
                };
                events_channel.send(e).unwrap();
            }
        });
    }
}
