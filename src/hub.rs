use std::collections::HashMap;
use std::sync::mpsc::channel;

use crate::channel;
use crate::config::Config;

pub struct Hub {
    config: Config,
    channels: HashMap<String, Box<dyn channel::Channel>>,
}

pub fn new(config: Config) -> Hub {
    // let slack = channel::slack::foo();
    let mut hub = Hub {
        config: config,
        channels: HashMap::new(),
    };

    for (name, cfg) in &hub.config.channels {
        let constructor = match cfg.class {
            channel::Type::SlackChannel => channel::slack::new,
        };

        let s = name.to_string();

        hub.channels.insert(s.clone(), constructor(s, cfg));
    }

    hub
}

impl Hub {
    pub fn run(&self) {
        info!("here we go!");

        let (tx, rx) = channel();

        let mut handles = vec![];
        for (_, c) in &self.channels {
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
