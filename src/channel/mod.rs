pub mod slack;

use std::sync::mpsc;

use crate::event;

pub trait Channel {
    fn start(&'static mut self, tx: mpsc::Sender<event::Event>);
}
