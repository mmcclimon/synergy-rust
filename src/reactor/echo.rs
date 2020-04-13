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

impl Reactor for Echo {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handlers_matching(&self, event: &Event) -> Vec<Handler> {
        vec![]
    }
}
