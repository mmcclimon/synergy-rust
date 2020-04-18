use std::sync::mpsc;
use std::thread;

use crate::hub::ReactorSeed;
use crate::message::{ReactorEvent, ReactorMessage, ReactorReply};

pub struct Echo {
    name: String,
    reply_tx: mpsc::Sender<ReactorReply>,
    event_rx: mpsc::Receiver<ReactorEvent>,
}

pub fn new(seed: ReactorSeed) -> Echo {
    Echo {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
    }
}

pub fn start(seed: ReactorSeed) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(seed);
        channel.start();
    });

    (name, handle)
}

impl Echo {
    fn start(&self) {
        for reactor_event in &self.event_rx {
            match reactor_event {
                ReactorEvent::Hangup => break,
                ReactorEvent::Message(event) => {
                    self.handle_echo(&event);
                }
            };
        }
    }

    fn send(&self, reply: ReactorReply) {
        self.reply_tx.send(reply).unwrap()
    }

    pub fn handle_echo(&self, event: &ReactorMessage) {
        let who = match &event.user {
            Some(u) => &u.username,
            None => "someone",
        };

        let text = format!("I heard {} say {}", who, event.text);

        self.send(event.reply(&text, &self.name));
    }
}
