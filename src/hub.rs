use std::collections::HashMap;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::channel::{self, ChannelConfig};
use crate::config::Config;
use crate::environment::{self, Environment};
use crate::message::{Event, Message};
use crate::reactor::{self, ReactorConfig};

pub struct Hub {
    child_handles: Vec<JoinHandle<()>>,
    channel_senders: HashMap<String, mpsc::Sender<Message>>,
    reactor_senders: Vec<mpsc::Sender<Message>>,
    reactor_count: u32,
    env: Option<Arc<Environment>>,

    // channels, which are useful to have as attributes
    channel_tx: mpsc::Sender<Message>,
    channel_rx: mpsc::Receiver<Message>,
    reactor_tx: mpsc::Sender<Message>,
    reactor_rx: mpsc::Receiver<Message>,
}

pub fn new() -> Hub {
    let (channel_tx, channel_rx) = mpsc::channel();
    let (reactor_tx, reactor_rx) = mpsc::channel();

    Hub {
        child_handles: vec![],
        reactor_senders: vec![],
        channel_senders: HashMap::new(),
        reactor_count: 0,
        env: None,

        channel_tx,
        channel_rx,
        reactor_tx,
        reactor_rx,
    }
}

#[derive(Debug)]
struct PendingReply {
    count: u32,
    will_respond: bool,
    event: Arc<Event>,
}

impl Hub {
    pub fn run(&mut self, config: Config) {
        info!("assembling hub");

        self.env = Some(environment::new(&config));

        self.assemble_reactors(config.reactors);
        self.assemble_channels(config.channels);

        self.listen();
    }

    pub fn listen(&mut self) {
        // id => pending
        let mut pending_replies: HashMap<String, PendingReply> = HashMap::new();

        loop {
            // write, then block on read.
            loop {
                match self.reactor_rx.try_recv() {
                    Ok(Message::Hangup) => self.shutdown(),
                    Ok(Message::Reply(reply)) => {
                        // figure out the destination, then send it along
                        // debug!("sending reply into channel");
                        let tx = self.channel_senders.get(&reply.destination).unwrap();
                        tx.send(Message::Reply(reply)).unwrap();
                    }
                    Ok(Message::Ack(id, this_resp)) => {
                        self.handle_ack(&mut pending_replies, id, this_resp);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("channel hung up on us??");
                    }
                    _ => (),
                }
            }

            // duration chosen by fair dice roll.
            match self.channel_rx.recv_timeout(Duration::from_millis(15)) {
                Ok(Message::Hangup) => self.shutdown(),
                Ok(Message::Event(channel_event)) => {
                    let event = self.transmogrify_event(channel_event);

                    // TODO this should be reference counted instead of cloned.
                    pending_replies.insert(
                        event.id.clone(),
                        PendingReply {
                            count: 0,
                            will_respond: false,
                            event: Arc::clone(&event),
                        },
                    );

                    // pass it along into reactors
                    for tx in &self.reactor_senders {
                        let clone = Arc::clone(&event);
                        tx.send(Message::Event(clone)).unwrap();
                    }
                }
                Ok(Message::Ack(_, _)) => panic!("events are not meant to send acks"),
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("channel hung up on us??"),
                _ => (),
            }
        }
    }

    fn handle_ack(
        &self,
        pending: &mut HashMap<String, PendingReply>,
        id: String,
        this_response: bool,
    ) {
        let r = pending.get_mut(&id).unwrap();
        r.count += 1;
        r.will_respond = this_response || r.will_respond;

        // hey, everyone has responded!
        if r.count == self.reactor_count {
            // if we were targeted and nobody wanted to respond, say something!
            if r.event.was_targeted && !r.will_respond {
                let reply = r.event.reply("Does not compute.", "hub");
                self.reactor_tx.send(reply).unwrap();
            }

            pending.remove(&id);
        }
    }

    fn assemble_channels(&mut self, channel_config: HashMap<String, ChannelConfig>) {
        for (raw_name, config) in channel_config {
            let name = format!("channel/{}", raw_name);
            info!("starting {}", name);

            // Hook up a line to this component. Into each channel we send:
            // 1. A transmitter (its output): we have one of these in our state,
            //    so we can just clone it.
            // 2. A receiver (its input): we'll keep track of its sending in in
            //    self.channel_senders
            let (this_tx, this_rx) = mpsc::channel();
            self.channel_senders.insert(name.clone(), this_tx);

            let handle = channel::build(name, config, self.channel_tx.clone(), this_rx);
            self.child_handles.push(handle);
        }
    }

    fn assemble_reactors(&mut self, reactor_config: HashMap<String, ReactorConfig>) {
        for (raw_name, config) in reactor_config {
            self.reactor_count += 1;

            let name = format!("reactor/{}", raw_name);
            info!("starting {}", name);

            let (this_tx, this_rx) = mpsc::channel();
            self.reactor_senders.push(this_tx);

            let handle = reactor::build(name, config, self.reactor_tx.clone(), this_rx);
            self.child_handles.push(handle);
        }
    }

    fn shutdown(&mut self) {
        // we ignore all errors here, because presumably they're just because
        // something has already hung up on us.
        info!("telling reactors to shut down...");
        for tx in self.reactor_senders.drain(..) {
            tx.send(Message::Hangup).unwrap_or(());
        }

        info!("telling channels to shut down...");
        for (_, tx) in self.channel_senders.drain() {
            tx.send(Message::Hangup).unwrap_or(());
        }

        info!("waiting for cleanup...");
        for handle in self.child_handles.drain(..) {
            handle.join().unwrap_or(());
        }

        info!("goodbye!");
        process::exit(0);
    }

    fn transmogrify_event(&self, orig: Arc<Event>) -> Arc<Event> {
        let user = self.env.as_ref().unwrap().resolve_user(&orig);
        let mut event = orig.dupe(); // silly, but ok
        event.user = user;
        Arc::new(event)
    }
}
