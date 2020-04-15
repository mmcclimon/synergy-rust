use std::sync::{mpsc, Arc};

use crate::channel::{self, ChannelConfig};
use crate::config::Config;
use crate::environment;
use crate::environment::Environment;
use crate::message::*; // {ChannelEvent, ReactorEvent};
use crate::reactor::{self, ReactorConfig};
use crate::user::MinimalUser;

pub struct Hub {
    // Almost certainly I want _something_ here, but not right now.
}

pub fn new() -> Hub {
    Hub {}
}

pub struct ChannelSeed {
    pub name: String,
    pub config: ChannelConfig,
    pub event_handle: mpsc::Sender<ChannelEvent>,
}

pub struct ReactorSeed {
    pub name: String,
    pub config: ReactorConfig,
    pub event_handle: mpsc::Receiver<Arc<ReactorEvent>>,
}

impl Hub {
    pub fn run(&self, config: Config) {
        info!("assembling hub");

        let env = environment::new(&config);

        let mut handles = vec![];
        let mut reactor_senders = vec![];

        let (channel_tx, channel_rx) = mpsc::channel();

        for (name, cfg) in config.channels {
            let starter = match cfg.class {
                channel::Type::SlackChannel => channel::slack::start,
            };

            let seed = ChannelSeed {
                name,
                config: cfg,
                event_handle: channel_tx.clone(),
            };

            let (addr, handle) = starter(seed);
            handles.push(handle);
            debug!("set up {}", addr);
        }

        for (name, cfg) in config.reactors {
            let starter = match cfg.class {
                reactor::Type::EchoReactor => reactor::echo::start,
            };

            let (reactor_tx, reactor_rx) = mpsc::channel();
            reactor_senders.push(reactor_tx);

            let seed = ReactorSeed {
                name,
                config: cfg,
                event_handle: reactor_rx,
            };

            let (addr, handle) = starter(seed);
            handles.push(handle);
            debug!("set up {}", addr);
        }

        for message in channel_rx {
            let reactor_event = Arc::new(self.transmogrify_event(message, &env));
            debug!("[hub] transmogged event: {:?}", reactor_event);

            for tx in &reactor_senders {
                let cloned = Arc::clone(&reactor_event);
                tx.send(cloned).unwrap();
            }

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
                let user = match env.resolve_user(&event) {
                    Some(u) => Some(MinimalUser::from(&u)),
                    None => None,
                };

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
