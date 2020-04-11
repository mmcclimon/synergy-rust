pub mod slack;

use std::sync::mpsc;

use crate::event;

pub trait Channel {
    fn start(&self, tx: mpsc::Sender<event::Event>);
}
