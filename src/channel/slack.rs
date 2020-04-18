mod client;

use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

use crate::channel::{Channel, ReplyResponse};
use crate::hub::ChannelSeed;
use crate::message::{ChannelEvent, ChannelMessage, ChannelReply, Reply};
use client::Client;

pub struct Slack {
    pub name: String,
    api_token: String,
    our_name: RefCell<Option<String>>,
    our_id: RefCell<Option<String>>,
    event_tx: mpsc::Sender<ChannelEvent>,
    reply_rx: mpsc::Receiver<ChannelReply>,
    rtm_client: Client,
}

// XXX clean me up

#[derive(Debug)]
struct SlackInternalError(String);

impl Error for SlackInternalError {}

impl fmt::Display for SlackInternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while talking to slack: {}", self.0)
    }
}

pub fn new(seed: ChannelSeed) -> Slack {
    let api_token = &seed.config.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    Slack {
        name: seed.name.clone(),
        api_token: api_token.to_string(),
        our_id: RefCell::new(None),
        our_name: RefCell::new(None),
        event_tx: seed.event_handle,
        reply_rx: seed.reply_handle,
        rtm_client: client::new(),
    }
}

pub fn start(seed: ChannelSeed) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(seed);
        channel.start();
    });

    (name, handle)
}

impl Channel for Slack {
    fn receiver(&self) -> &mpsc::Receiver<ChannelReply> {
        &self.reply_rx
    }

    fn send_reply(&self, reply: Reply) {
        self.rtm_client.send(reply);
    }
}

impl Slack {
    fn start(&self) {
        let me = self.rtm_client.connect(&self.api_token);

        self.our_name.replace(Some(me.name));
        self.our_id.replace(Some(me.id));

        loop {
            match self.catch_replies() {
                ReplyResponse::Hangup => break,
                _ => (),
            };

            let raw_event = match self.rtm_client.recv() {
                Some(raw) => raw,
                None => continue,
            };

            let msg = ChannelEvent::Message(ChannelMessage {
                // TODO: fill these in properly
                text: raw_event.text,
                is_public: false,
                was_targeted: true,
                from_address: raw_event.user,
                conversation_address: raw_event.channel,
                origin: self.name.clone(),
            });

            self.event_tx.send(msg).unwrap();
        }
    }
}
