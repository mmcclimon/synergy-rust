use std::sync::mpsc;
use std::thread;

use crate::message::{Event, Message, Reply};
use crate::reactor::{Handler, Reactor, Seed};

pub struct Echo {
    name: String,
    reply_tx: mpsc::Sender<Message<Reply>>,
    event_rx: mpsc::Receiver<Message<Event>>,
    handlers: Vec<Handler<Dispatch>>,
}

enum Dispatch {
    HandleEcho,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Echo {
    Echo {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
        handlers: vec![Handler {
            require_targeted: true,
            predicate: |_| true,
            magic: Dispatch::HandleEcho,
        }],
    }
}

impl Reactor<Dispatch> for Echo {
    fn name(&self) -> &str {
        &self.name
    }

    fn handlers(&self) -> &Vec<Handler<Dispatch>> {
        &self.handlers
    }

    fn event_rx(&self) -> &mpsc::Receiver<Message<Event>> {
        &self.event_rx
    }

    fn reply_tx(&self) -> &mpsc::Sender<Message<Reply>> {
        &self.reply_tx
    }

    fn dispatch(&self, magic: &Dispatch, event: &Event) {
        match magic {
            Dispatch::HandleEcho => self.handle_echo(&event),
        };
    }
}

impl Echo {
    pub fn handle_echo(&self, event: &Event) {
        let who = match &event.user {
            Some(u) => &u.username,
            None => "someone",
        };

        let text = format!("I heard {} say {}", who, event.text);
        self.reply_to(&event, &text);
    }
}
