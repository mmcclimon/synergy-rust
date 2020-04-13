use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

use crate::channel;
use crate::config::Config;
use crate::environment;

pub struct Hub {
    // config: Config,
    channels: HashMap<String, Box<dyn channel::Channel>>,
    environment: Rc<environment::Environment>,
}

pub fn new(config: Config) -> Hub {
    // let slack = channel::slack::foo();
    // config: config,
    let mut hub = Hub {
        channels: HashMap::new(),
        environment: environment::new(&config),
    };

    for (name, cfg) in &config.channels {
        let constructor = match cfg.class {
            channel::Type::SlackChannel => channel::slack::new,
        };

        let s = name.to_string();

        hub.channels.insert(
            s.clone(),
            constructor(s, cfg, Rc::downgrade(&hub.environment)),
        );
    }

    hub
}

impl Hub {
    pub fn run(&self) {
        info!("here we go!");

        let (tx, rx) = mpsc::channel();

        let mut handles = vec![];
        for c in self.channels.values() {
            let event_channel = tx.clone();
            let handle = c.start(event_channel);
            handles.push(handle);
        }

        for event in rx {
            debug!("[hub] got event: {:?}", event);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
