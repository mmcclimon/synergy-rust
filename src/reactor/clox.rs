use std::thread;

use crate::message::Event;
use crate::reactor::{Core, Handler, Reactor, Seed};

pub struct Clox {
    core: Core<Dispatch>,
}

pub enum Dispatch {
    HandleClox,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Clox {
    let core = Core {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
        handlers: vec![Handler {
            predicate: |event| event.text.starts_with("clox"),
            require_targeted: true,
            key: Dispatch::HandleClox,
        }],
    };

    Clox { core }
}

impl Reactor for Clox {
    type Dispatcher = Dispatch;

    fn core(&self) -> &Core<Dispatch> {
        &self.core
    }

    fn dispatch(&self, key: &Dispatch, event: &Event) {
        match key {
            Dispatch::HandleClox => self.handle_clox(&event),
        };
    }
}

impl Clox {
    fn handle_clox(&self, event: &Event) {
        let text = format!("would handle clox");
        self.reply_to(&event, &text);
    }
}
