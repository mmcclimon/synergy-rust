use std::collections::HashMap;
use std::sync::{mpsc, Arc, RwLock};

use crate::channel;
use crate::config::Config;
use crate::environment;

pub struct Hub {
    // config: Config,
    channels: RwLock<HashMap<String, Arc<dyn channel::Channel>>>,
    environment: Arc<environment::Environment>,
}

pub fn new(config: Config) -> Arc<Hub> {
    // let slack = channel::slack::foo();
    // config: config,
    let hub = Arc::new(Hub {
        channels: RwLock::new(HashMap::new()),
        environment: environment::new(&config),
    });

    for (name, cfg) in &config.channels {
        let constructor = match cfg.class {
            channel::Type::SlackChannel => channel::slack::new,
        };

        let s = name.to_string();

        let mut channels = hub.channels.write().unwrap();
        channels.insert(s.clone(), constructor(s, cfg, Arc::downgrade(&hub)));
    }

    hub
}

impl Hub {
    pub fn run(&self) {
        info!("here we go!");

        let (tx, rx) = mpsc::channel();

        let mut handles = vec![];
        for c in self.channels.read().unwrap().values() {
            let channel = Arc::clone(&c);
            let event_channel = tx.clone();
            let handle = channel.start(event_channel);
            handles.push(handle);
        }

        for mut event in rx {
            debug!("[hub] got event: {:?}", event);
            event.ensure_complete(&self.environment);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
