pub mod clox;
pub mod echo;

use std::sync::mpsc;
use std::thread;

use serde::Deserialize;

use crate::config::ComponentConfig;
use crate::message::{Event, Message, Reply};

// known reactors
#[derive(Deserialize, Debug)]
pub enum Type {
    EchoReactor,
    CloxReactor,
}

pub type ReactorConfig = ComponentConfig<Type>;

pub struct Seed {
    pub name: String,
    pub config: ReactorConfig,
    pub reply_handle: mpsc::Sender<Message<Reply>>,
    pub event_handle: mpsc::Receiver<Message<Event>>,
}

pub fn build(
    name: String,
    config: ReactorConfig,
    reply_handle: mpsc::Sender<Message<Reply>>,
    event_handle: mpsc::Receiver<Message<Event>>,
) -> thread::JoinHandle<()> {
    let builder = match config.class {
        Type::EchoReactor => echo::build,
        Type::CloxReactor => clox::build,
    };

    let seed = Seed {
        name,
        config,
        reply_handle,
        event_handle,
    };

    builder(seed)
}

// Is this abstraction _just_ for the pun? Not quite!
pub struct Core<D> {
    name: String,
    reply_tx: mpsc::Sender<Message<Reply>>,
    event_rx: mpsc::Receiver<Message<Event>>,
    handlers: Vec<Handler<D>>,
}

pub struct Handler<T> {
    predicate: fn(&Event) -> bool,
    require_targeted: bool,
    key: T,
}

impl<T> Handler<T> {
    pub fn matches(&self, e: &Event) -> bool {
        (self.predicate)(e)
    }
}

impl<T> Core<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn handlers(&self) -> &Vec<Handler<T>> {
        &self.handlers
    }

    fn event_rx(&self) -> &mpsc::Receiver<Message<Event>> {
        &self.event_rx
    }

    fn reply_tx(&self) -> &mpsc::Sender<Message<Reply>> {
        &self.reply_tx
    }
}

pub trait Reactor {
    type Dispatcher;

    fn core(&self) -> &Core<Self::Dispatcher>;
    fn dispatch(&self, key: &Self::Dispatcher, event: &Event);

    fn start(&mut self) {
        for reactor_event in self.core().event_rx() {
            match reactor_event {
                Message::Hangup => break,
                Message::Text(event) => self.dispatch_event(&event),
            };
        }
    }

    fn dispatch_event(&self, event: &Event) {
        for handler in self.core().handlers() {
            if handler.require_targeted && !event.was_targeted {
                continue;
            }

            if handler.matches(&event) {
                self.dispatch(&handler.key, &event);
            }
        }
    }

    fn reply_to(&self, event: &Event, text: &str) {
        let reply = event.reply(text, self.core().name());
        self.core().reply_tx().send(reply).unwrap();
    }
}
