use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::channel::{self, ChannelConfig};
use crate::config::Config;
use crate::environment::{self, Environment};
use crate::message::*; // {ChannelEvent, ReactorEvent};
use crate::reactor::{self, ReactorConfig};

pub struct Hub {
    // Almost certainly I want _something_ here, but not right now.
    child_handles: Vec<JoinHandle<()>>,
    channel_senders: HashMap<String, mpsc::Sender<ChannelReply>>,
    reactor_senders: Vec<mpsc::Sender<ReactorEvent>>,
    env: Option<Arc<Environment>>,
}

pub fn new() -> Hub {
    Hub {
        child_handles: vec![],
        reactor_senders: vec![],
        channel_senders: HashMap::new(),
        env: None,
    }
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
    pub fn run(&mut self, config: Config) {
        info!("assembling hub");

        self.env = Some(environment::new(&config));

        let (event_tx, event_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();

        for (raw_name, cfg) in config.channels {
            let starter = match cfg.class {
                channel::Type::SlackChannel => channel::slack::start,
                channel::Type::TermChannel => channel::term::start,
            };

            let name = format!("channel/{}", raw_name);
            info!("starting {}", name);

            // we have to send a receiver into the channel, and keep track of
            // our senders
            let (channel_tx, channel_rx) = mpsc::channel();
            self.channel_senders.insert(name.clone(), channel_tx);

            let seed = ChannelSeed {
                name,
                config: cfg,
                event_handle: event_tx.clone(),
                reply_handle: channel_rx,
            };

            let (_addr, handle) = starter(seed);
            self.child_handles.push(handle);
        }

        for (raw_name, cfg) in config.reactors {
            let starter = match cfg.class {
                reactor::Type::EchoReactor => reactor::echo::start,
            };

            let name = format!("reactor/{}", raw_name);
            info!("starting {}", name);

            let (reactor_tx, reactor_rx) = mpsc::channel();
            self.reactor_senders.push(reactor_tx);

            let seed = ReactorSeed {
                name,
                config: cfg,
                event_handle: reactor_rx,
                reply_handle: reply_tx.clone(),
            };

            let (_addr, handle) = starter(seed);
            self.child_handles.push(handle);
        }

        loop {
            // write, then block on read.
            loop {
                match reply_rx.try_recv() {
                    Ok(ReactorReply::Message(reply)) => {
                        // figure out the destination, then send it along
                        // debug!("sending reply into channel");
                        let tx = self.channel_senders.get(&reply.destination).unwrap();
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
                Ok(ChannelEvent::Hangup) => self.shutdown(),
                Ok(ChannelEvent::Message(message)) => {
                    let reactor_event = self.transmogrify_message(message);

                    // debug!("sending event into reactors");

                    // pass it along into reactors
                    for tx in &self.reactor_senders {
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

    fn shutdown(&self) {
        warn!("need shutdown code!");
    }

    fn transmogrify_message(&self, event: ChannelMessage) -> ReactorEvent {
        let user = self.env.as_ref().unwrap().resolve_user(&event);
        ReactorEvent::Message(ReactorMessage {
            text: event.text,
            is_public: event.is_public,
            was_targeted: event.was_targeted,
            from_address: event.from_address,
            conversation_address: event.conversation_address,
            origin: event.origin,
            user: user,
        })
    }
}
