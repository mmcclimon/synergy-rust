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

pub trait Reactor {
    // TODO
}
