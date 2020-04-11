use std::sync::mpsc::channel;

use crate::channel;

pub struct Hub {
    name: String,
    channels: Vec<Box<dyn channel::Channel>>,
}

pub fn new(name: &str) -> Hub {
    // let slack = channel::slack::foo();
    let mut hub = Hub {
        name: name.to_string(),
        channels: vec![],
    };

    hub.channels.push(channel::slack::new());

    hub
}

impl Hub {
    pub fn run(&self) {
        info!("running things from hub named {}", self.name);

        let (tx, rx) = channel();

        let mut handles = vec![];
        for c in &self.channels {
            let event_channel = tx.clone();
            let handle = c.start(event_channel);
            handles.push(handle);
        }

        for event in rx {
            debug!("got event: {:?}", event);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
