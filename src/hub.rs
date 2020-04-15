use std::collections::HashMap;
use std::sync::{mpsc, Arc, RwLock};

use crate::channel;
use crate::config::{ComponentConfig, Config};
use crate::environment;
use crate::environment::Environment;
use crate::message::*; // {ChannelEvent, ReactorEvent};
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

        let env = environment::new(&config);

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
            let reactor_event = self.transmogrify_event(message, &env);
            debug!("[hub] transmogged event: {:?}", reactor_event);

            // send into reactors
        }

        // loop forever
        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn transmogrify_event(&self, channel_event: ChannelEvent, env: &Environment) -> ReactorEvent {
        let msg = match channel_event {
            ChannelEvent::Message(event) => {
                let user = env.resolve_user(&event);
                ReactorMessage {
                    text: event.text,
                    is_public: event.is_public,
                    was_targeted: event.was_targeted,
                    from_address: event.from_address,
                    conversation_address: event.conversation_address,
                    origin: event.origin,
                    user: user,
                }
            }
        };

        ReactorEvent::Message(msg)
    }
}
