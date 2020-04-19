use std::sync::mpsc;
use std::thread;

use crate::message::{Event, Message, Reply};
use crate::reactor::Seed;

pub struct Echo {
    name: String,
    reply_tx: mpsc::Sender<Message<Reply>>,
    event_rx: mpsc::Receiver<Message<Event>>,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Echo {
    Echo {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
    }
}

impl Echo {
    fn start(&self) {
        for reactor_event in &self.event_rx {
            match reactor_event {
                Message::Hangup => break,
                Message::Text(event) => {
                    self.handle_echo(&event);
                }
            };
        }
    }

    fn send(&self, reply: Message<Reply>) {
        self.reply_tx.send(reply).unwrap()
    }

    pub fn handle_echo(&self, event: &Event) {
        let who = match &event.user {
            Some(u) => &u.username,
            None => "someone",
        };

        if !event.was_targeted {
            return;
        }

        let text = format!("I heard {} say {}", who, event.text);

        self.send(event.reply(&text, &self.name));
    }
}
