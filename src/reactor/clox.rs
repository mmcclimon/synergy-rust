use std::sync::mpsc;
use std::thread;

use crate::message::{Event, Message, Reply};
use crate::reactor::Seed;

pub struct Clox {
    name: String,
    reply_tx: mpsc::Sender<Message<Reply>>,
    event_rx: mpsc::Receiver<Message<Event>>,
    handlers: Vec<Handler>,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Clox {
    Clox {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
        handlers: vec![],
    }
}

pub struct Handler {
    name: String,
    predicate: fn(&Event) -> bool,
    method: fn(&Clox, &Event) -> (),
    require_targeted: bool,
}

impl Clox {
    fn start(&mut self) {
        let h = Handler {
            name: String::from("clox"),
            predicate: |event| event.text.starts_with("clox"),
            method: handle_clox,
            require_targeted: true,
        };

        self.handlers.push(h);

        for reactor_event in &self.event_rx {
            match reactor_event {
                Message::Hangup => break,
                Message::Text(event) => {
                    for h in &self.handlers {
                        if (h.predicate)(&event) {
                            debug!("match");
                            (h.method)(&self, &event);
                        }
                    }
                }
            };
        }
    }

    fn send(&self, reply: Message<Reply>) {
        self.reply_tx.send(reply).unwrap()
    }
}

fn handle_clox(clox: &Clox, event: &Event) {
    if !event.was_targeted {
        return;
    }

    let text = format!("would handle clox");

    clox.send(event.reply(&text, &clox.name));
}
