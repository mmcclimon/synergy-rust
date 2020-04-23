pub mod slack;
pub mod term;

use std::sync::{mpsc, Arc};
use std::thread;

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

pub fn build(
    name: String,
    config: ChannelConfig,
    event_handle: mpsc::Sender<Message<Event>>,
    reply_handle: mpsc::Receiver<Message<Reply>>,
) -> thread::JoinHandle<()> {
    let builder = match config.class {
        Type::SlackChannel => slack::build,
        Type::TermChannel => term::build,
    };

    let seed = Seed {
        name,
        config,
        reply_handle,
        event_handle,
    };

    builder(seed)
}

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

    fn send_reply(&mut self, r: Arc<Reply>);

    fn catch_replies(&mut self) -> ReplyResponse {
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
                _ => (),
            }
        }

        if did_send {
            ReplyResponse::Sent
        } else {
            ReplyResponse::Empty
        }
    }
}
