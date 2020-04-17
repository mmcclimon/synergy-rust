pub mod slack;
pub mod term;

use std::sync::mpsc;
use std::thread;

use serde::Deserialize;

use crate::config;
use crate::event;

// known channels
#[derive(Deserialize, Debug)]
pub enum Type {
    SlackChannel,
    TermChannel,
}

pub type ChannelConfig = config::ComponentConfig<Type>;

pub trait Channel {
    fn start(&self, tx: mpsc::Sender<event::Event>) -> thread::JoinHandle<()>;

    fn name(&self) -> String;
}
