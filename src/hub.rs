use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use crate::channel::{self, ChannelConfig};
use crate::config::Config;
use crate::environment;
use crate::environment::Environment;
use crate::message::*; // {ChannelEvent, ReactorEvent};
use crate::reactor::{self, ReactorConfig};

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
    pub reply_handle: mpsc::Receiver<ChannelReply>,
}

pub struct ReactorSeed {
    pub name: String,
    pub config: ReactorConfig,
    pub event_handle: mpsc::Receiver<ReactorEvent>,
    pub reply_handle: mpsc::Sender<ReactorReply>,
}

impl Hub {
    pub fn run(&self, config: Config) {
        info!("assembling hub");

        let env = environment::new(&config);

        let mut handles = vec![];
        let mut reactor_senders = vec![];
        let mut channel_senders = HashMap::new();

        let (event_tx, event_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();

        for (raw_name, cfg) in config.channels {
            let starter = match cfg.class {
                channel::Type::SlackChannel => channel::slack::start,
            };

            let name = format!("channel/{}", raw_name);
            info!("starting {}", name);

            // we have to send a receiver into the channel, and keep track of
            // our senders
            let (channel_tx, channel_rx) = mpsc::channel();
            channel_senders.insert(name.clone(), channel_tx);

            let seed = ChannelSeed {
                name,
                config: cfg,
                event_handle: event_tx.clone(),
                reply_handle: channel_rx,
            };

            let (_addr, handle) = starter(seed);
            handles.push(handle);
        }

        for (raw_name, cfg) in config.reactors {
            let starter = match cfg.class {
                reactor::Type::EchoReactor => reactor::echo::start,
            };

            let name = format!("reactor/{}", raw_name);
            info!("starting {}", name);

            let (reactor_tx, reactor_rx) = mpsc::channel();
            reactor_senders.push(reactor_tx);

            let seed = ReactorSeed {
                name,
                config: cfg,
                event_handle: reactor_rx,
                reply_handle: reply_tx.clone(),
            };

            let (_addr, handle) = starter(seed);
            handles.push(handle);
        }

        loop {
            // write, then block on read.
            loop {
                match reply_rx.try_recv() {
                    Ok(ReactorReply::Message(reply)) => {
                        // figure out the destination, then send it along
                        // debug!("sending reply into channel");
                        let tx = channel_senders.get(&reply.destination).unwrap();
                        tx.send(ChannelReply::Message(reply)).unwrap();
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("channel hung up on us??");
                    }
                }
            }

            // duration chosen by fair dice roll.
            match event_rx.recv_timeout(Duration::from_millis(15)) {
                Ok(message) => {
                    let reactor_event = self.transmogrify_event(message, &env);

                    // debug!("sending event into reactors");

                    // pass it along into reactors
                    for tx in &reactor_senders {
                        let cloned = reactor_event.clone();
                        tx.send(cloned).unwrap();
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("channel hung up on us??"),
            }
        }

        // this code joins threads, but will never run because of the loop above
        // for handle in handles { handle.join().unwrap() }
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
