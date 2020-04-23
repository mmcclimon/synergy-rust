use std::collections::HashMap;
use std::process;
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::channel::{self, ChannelConfig};
use crate::config::Config;
use crate::environment::{self, Environment};
use crate::message::{Event, Message, Reply};
use crate::reactor::{self, ReactorConfig};

pub struct Hub {
    // Almost certainly I want _something_ here, but not right now.
    child_handles: Vec<JoinHandle<()>>,
    channel_senders: HashMap<String, mpsc::Sender<Message<Reply>>>,
    reactor_senders: Vec<mpsc::Sender<Message<Event>>>,
    env: Option<Arc<Environment>>,

    // machinery for handling Does not compute.
    reply_tx: Option<mpsc::Sender<Message<Reply>>>,
    reactor_count: u32,
}

pub fn new() -> Hub {
    Hub {
        child_handles: vec![],
        reactor_senders: vec![],
        channel_senders: HashMap::new(),
        reactor_count: 0,
        reply_tx: None,
        env: None,
    }
}

#[derive(Debug)]
struct PendingReply {
    count: u32,
    will_respond: bool,
    event: Event,
}

impl Hub {
    pub fn run(&mut self, config: Config) {
        info!("assembling hub");

        self.env = Some(environment::new(&config));

        let (event_tx, event_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();

        self.reply_tx = Some(reply_tx.clone());

        // Send the sending end into the channel/reactor as appropriate. These
        // methods set up the other direction, and store themselves as state
        self.assemble_reactors(reply_tx, config.reactors);
        self.assemble_channels(event_tx, config.channels);

        self.listen(event_rx, reply_rx);
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
                self.reply_tx.as_ref().unwrap().send(reply).unwrap();
            }

            pending.remove(&id);
        }
    }

    pub fn listen(
        &mut self,
        event_rx: mpsc::Receiver<Message<Event>>,
        reply_rx: mpsc::Receiver<Message<Reply>>,
    ) {
        // id => pending
        let mut pending_replies: HashMap<String, PendingReply> = HashMap::new();

        loop {
            // write, then block on read.
            loop {
                match reply_rx.try_recv() {
                    Ok(Message::Hangup) => self.shutdown(),
                    Ok(Message::Text(reply)) => {
                        // figure out the destination, then send it along
                        // debug!("sending reply into channel");
                        let tx = self.channel_senders.get(&reply.destination).unwrap();
                        tx.send(Message::Text(reply)).unwrap();
                    }
                    Ok(Message::Ack(id, this_resp)) => {
                        self.handle_ack(&mut pending_replies, id, this_resp);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("channel hung up on us??");
                    }
                }
            }

            // duration chosen by fair dice roll.
            match event_rx.recv_timeout(Duration::from_millis(15)) {
                Ok(Message::Hangup) => self.shutdown(),
                Ok(Message::Text(ref mut event)) => {
                    self.transmogrify_event(event);

                    // TODO this should be reference counted instead of cloned.
                    pending_replies.insert(
                        event.id.clone(),
                        PendingReply {
                            count: 0,
                            will_respond: false,
                            event: event.clone(),
                        },
                    );

                    // pass it along into reactors
                    for tx in &self.reactor_senders {
                        let cloned = event.clone();
                        tx.send(Message::Text(cloned)).unwrap();
                    }
                }
                Ok(Message::Ack(_, _)) => panic!("events are not meant to send acks"),
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("channel hung up on us??"),
            }
        }
    }

    fn assemble_channels(
        &mut self,
        event_tx: mpsc::Sender<Message<Event>>,
        channel_config: HashMap<String, ChannelConfig>,
    ) {
        for (raw_name, config) in channel_config {
            let name = format!("channel/{}", raw_name);
            info!("starting {}", name);

            // we have to send a receiver into the channel, and keep track of
            // our senders
            let (channel_tx, channel_rx) = mpsc::channel();
            self.channel_senders.insert(name.clone(), channel_tx);

            let handle = channel::build(name, config, event_tx.clone(), channel_rx);
            self.child_handles.push(handle);
        }
    }

    fn assemble_reactors(
        &mut self,
        reply_tx: mpsc::Sender<Message<Reply>>,
        reactor_config: HashMap<String, ReactorConfig>,
    ) {
        for (raw_name, config) in reactor_config {
            self.reactor_count += 1;

            let name = format!("reactor/{}", raw_name);
            info!("starting {}", name);

            let (reactor_tx, reactor_rx) = mpsc::channel();
            self.reactor_senders.push(reactor_tx);

            let handle = reactor::build(name, config, reply_tx.clone(), reactor_rx);
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

    fn transmogrify_event(&self, event: &mut Event) {
        let user = self.env.as_ref().unwrap().resolve_user(&event);
        event.user = user;
    }
}
