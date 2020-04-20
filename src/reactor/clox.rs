use std::sync::mpsc;
use std::thread;

use crate::message::{Event, Message, Reply};
use crate::reactor::{Handler, Reactor, Seed};

pub struct Clox {
    name: String,
    reply_tx: mpsc::Sender<Message<Reply>>,
    event_rx: mpsc::Receiver<Message<Event>>,
    handlers: Vec<Handler<Dispatch>>,
}

enum Dispatch {
    HandleClox,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Clox {
    let mut reactor = Clox {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
        handlers: vec![],
    };

    // add our handlers
    reactor.handlers.push(Handler {
        predicate: |event| event.text.starts_with("clox"),
        require_targeted: true,
        magic: Dispatch::HandleClox,
    });

    reactor
}

impl Reactor<Dispatch> for Clox {
    fn handlers(&self) -> &Vec<Handler<Dispatch>> {
        &self.handlers
    }

    fn event_rx(&self) -> &mpsc::Receiver<Message<Event>> {
        &self.event_rx
    }

    fn dispatch(&self, thing: &Dispatch, event: &Event) {
        match thing {
            Dispatch::HandleClox => self.handle_clox(&event),
        };
    }
}

impl Clox {
    fn send(&self, reply: Message<Reply>) {
        self.reply_tx.send(reply).unwrap()
    }

    fn handle_clox(&self, event: &Event) {
        if !event.was_targeted {
            return;
        }

        let text = format!("would handle clox");

        self.send(event.reply(&text, &self.name));
    }
}
