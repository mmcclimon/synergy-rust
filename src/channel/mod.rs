pub mod slack;
pub mod term;

use std::sync::mpsc;

use serde::Deserialize;

use crate::config;
use crate::message::{Event, Message, Reply};

// known channels
#[derive(Deserialize, Debug)]
pub enum Type {
    SlackChannel,
    TermChannel,
}

pub type ChannelConfig = config::ComponentConfig<Type>;

// stupid name
pub enum ReplyResponse {
    Hangup,
    Empty,
    Sent,
}

pub struct Seed {
    pub name: String,
    pub config: ChannelConfig,
    pub event_handle: mpsc::Sender<Message<Event>>,
    pub reply_handle: mpsc::Receiver<Message<Reply>>,
}

pub trait Channel {
    fn receiver(&self) -> &mpsc::Receiver<Message<Reply>>;

    fn send_reply(&self, r: Reply);

    fn catch_replies(&self) -> ReplyResponse {
        let mut did_send = false;

        loop {
            match self.receiver().try_recv() {
                Ok(Message::Hangup) => return ReplyResponse::Hangup,
                Ok(Message::Text(reply)) => {
                    self.send_reply(reply);
                    did_send = true;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("hub hung up on us?");
                }
            }
        }

        if did_send {
            ReplyResponse::Sent
        } else {
            ReplyResponse::Empty
        }
    }
}
