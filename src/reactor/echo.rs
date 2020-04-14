use std::sync::{Arc, Weak};

use crate::event::Event;
use crate::hub::Hub;
use crate::reactor::{Handler, Reactor, ReactorConfig};

pub struct Echo {
    name: String,
}

pub fn new(name: String, _cfg: &ReactorConfig, _hub: Weak<Hub>) -> Arc<Echo> {
    Arc::new(Echo { name })
}

impl Echo {
    pub fn handle_echo(&self, event: &Event) {
        debug!(
            "got event from {}: {}",
            event.from_user.as_ref().unwrap().username,
            event.text
        );

        event.reply(&event.text);
    }
}

impl Reactor for Echo {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn react_to(&self, event: &Event) {
        self.handle_echo(event);
    }
}
