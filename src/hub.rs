use std::sync::{mpsc, Arc};
use std::time::Duration;

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
    pub reply_handle: mpsc::Receiver<ChannelReply>,
}

pub struct ReactorSeed {
    pub name: String,
    pub config: ReactorConfig,
    pub event_handle: mpsc::Receiver<Arc<ReactorEvent>>,
    pub reply_handle: mpsc::Sender<ReactorReply>,
}

impl Hub {
    pub fn run(&self, config: Config) {
        info!("assembling hub");

        let env = environment::new(&config);

        let mut handles = vec![];
        let mut reactor_senders = vec![];
        let mut channel_senders = vec![];

        let (event_tx, event_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();

        for (name, cfg) in config.channels {
            let starter = match cfg.class {
                channel::Type::SlackChannel => channel::slack::start,
            };

            // we have to send a receiver into the channel, and keep track of
            // our senders
            let (channel_tx, channel_rx) = mpsc::channel();
            channel_senders.push(channel_tx);

            let seed = ChannelSeed {
                name,
                config: cfg,
                event_handle: event_tx.clone(),
                reply_handle: channel_rx,
            };

            let (addr, handle) = starter(seed);
            handles.push(handle);
            debug!("set up {}", addr);
        }

        for (name, cfg) in config.reactors {
            let starter = match cfg.class {
                reactor::Type::EchoReactor => reactor::echo::start,
            };

            // eent
            let (reactor_tx, reactor_rx) = mpsc::channel();
            reactor_senders.push(reactor_tx);

            let seed = ReactorSeed {
                name,
                config: cfg,
                event_handle: reactor_rx,
                reply_handle: reply_tx.clone(),
            };

            let (addr, handle) = starter(seed);
            handles.push(handle);
            debug!("set up {}", addr);
        }

        loop {
            // write, then block on read.
            loop {
                match reply_rx.try_recv() {
                    Ok(message) => {
                        debug!("got reply, must handle: {:?}", message);

                        // look up its destination, pass it along.
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("channel hung up on us??");
                    }
                }
            }

            // duration chosen by fair dice roll.
            match event_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(message) => {
                    let reactor_event = Arc::new(self.transmogrify_event(message, &env));
                    debug!("[hub] transmogged event: {:?}", reactor_event);

                    // pass it along into reactors
                    for tx in &reactor_senders {
                        let cloned = Arc::clone(&reactor_event);
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
