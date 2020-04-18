pub mod slack;
pub mod term;

use std::sync::mpsc;
use std::thread;

use serde::Deserialize;

use crate::config;
use crate::event;
use crate::message::{ChannelEvent, ChannelMessage, ChannelReply, Reply};

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

pub trait Channel {
    fn receiver(&self) -> &mpsc::Receiver<ChannelReply>;

    fn send_reply(&self, r: Reply);

    fn catch_replies(&self) -> ReplyResponse {
        let mut did_send = false;

        loop {
            match self.receiver().try_recv() {
                Ok(ChannelReply::Hangup) => return ReplyResponse::Hangup,
                Ok(ChannelReply::Message(reply)) => {
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
