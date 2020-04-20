pub mod clox;
pub mod echo;

use std::sync::mpsc;

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
    pub event_handle: mpsc::Receiver<Message<Event>>,
    pub reply_handle: mpsc::Sender<Message<Reply>>,
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

pub trait Reactor<T> {
    fn name(&self) -> &str;
    fn handlers(&self) -> &Vec<Handler<T>>;
    fn event_rx(&self) -> &mpsc::Receiver<Message<Event>>;
    fn reply_tx(&self) -> &mpsc::Sender<Message<Reply>>;
    fn dispatch(&self, key: &T, event: &Event);

    fn start(&mut self) {
        for reactor_event in self.event_rx() {
            match reactor_event {
                Message::Hangup => break,
                Message::Text(event) => self.check_event(&event),
            };
        }
    }

    fn check_event(&self, event: &Event) {
        for handler in self.handlers() {
            if handler.require_targeted && !event.was_targeted {
                continue;
            }

            if handler.matches(&event) {
                self.dispatch(&handler.key, &event);
            }
        }
    }

    fn reply_to(&self, event: &Event, text: &str) {
        let reply = event.reply(text, self.name());
        self.reply_tx().send(reply).unwrap();
    }
}
