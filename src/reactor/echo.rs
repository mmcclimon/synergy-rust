use std::sync::{mpsc, Arc};
use std::thread;

use crate::hub::ReactorSeed;
use crate::message::{ReactorEvent, ReactorMessage};

pub struct Echo {
    _name: String,
}

pub fn new(seed: &ReactorSeed) -> Echo {
    Echo {
        _name: seed.name.clone(),
    }
}

pub fn start(seed: ReactorSeed) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(&seed);
        channel.start(seed.event_handle);
    });

    (name, handle)
}

impl Echo {
    fn start(&self, events_channel: mpsc::Receiver<Arc<ReactorEvent>>) {
        use std::borrow::Borrow;

        for reactor_event in events_channel {
            match reactor_event.borrow() {
                ReactorEvent::Message(event) => {
                    self.handle_echo(&event);
                }
            };
        }
    }

    pub fn handle_echo(&self, event: &ReactorMessage) {
        debug!(
            "got event from {}: {}",
            event.user.as_ref().unwrap().username,
            event.text
        );

        // event.reply(&event.text);
    }
}
