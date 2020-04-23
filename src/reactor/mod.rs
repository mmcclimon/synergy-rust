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
    will_respond: bool,
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
                _ => (),
            };
        }
    }

    fn dispatch_event(&self, event: &Event) {
        let mut matched_keys = vec![];
        let mut will_respond = false;

        // So, when we catch an event, we will immediately check all the
        // handlers and see if they'll respond, so that we can send an ack to
        // the hub. Once we've done that, we'll actually run the methods, which
        // can run at their leisure.
        for handler in self.core().handlers() {
            if handler.require_targeted && !event.was_targeted {
                continue;
            }

            if handler.matches(&event) {
                matched_keys.push(&handler.key);
                if handler.will_respond {
                    will_respond = true;
                }
            }
        }

        self.ack(&event.id, will_respond);

        // now dispatch
        for key in &matched_keys {
            self.dispatch(&key, &event);
        }
    }

    fn send_reply_to_hub(&self, msg: Message<Reply>) {
        self.core().reply_tx().send(msg).unwrap();
    }

    fn ack(&self, id: &str, will_respond: bool) {
        self.send_reply_to_hub(Message::Ack(String::from(id), will_respond));
    }

    fn reply_to(&self, event: &Event, text: &str) {
        let reply = event.reply(text, self.core().name());
        self.send_reply_to_hub(reply);
    }
}
