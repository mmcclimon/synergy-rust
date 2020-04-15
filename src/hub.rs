use std::collections::HashMap;
use std::sync::{mpsc, Arc, RwLock};

use crate::channel;
use crate::config::{ComponentConfig, Config};
use crate::environment;
use crate::message::ChannelEvent;
use crate::reactor;

pub struct Hub {
    // Almost certainly I want _something_ here, but not right now.
}

pub fn new() -> Hub {
    Hub {}
}

pub struct Seed<T> {
    pub name: String,
    pub config: T,
    pub event_handle: mpsc::Sender<ChannelEvent>,
}

impl Hub {
    pub fn run(&self, config: Config) {
        info!("assembling hub");

        let mut handles = vec![];

        let (channel_tx, channel_rx) = mpsc::channel();

        for (name, cfg) in config.channels {
            let starter = match cfg.class {
                channel::Type::SlackChannel => channel::slack::start,
            };

            let seed = Seed {
                name,
                config: cfg,
                event_handle: channel_tx.clone(),
            };

            let (addr, handle) = starter(seed);
            handles.push(handle);
            debug!("set up {}", addr);
        }

        for message in channel_rx {
            // event.ensure_complete(&self.environment);
            debug!("[hub] got event: {:?}", message);

            // assemble message, send into reactors
        }

        // loop forever
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
