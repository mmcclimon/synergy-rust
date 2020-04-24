pub mod slack;
pub mod term;

use std::sync::mpsc;
use std::thread;

use serde::Deserialize;

use crate::config;
use crate::message::{Message, Reply};

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
    output: mpsc::Sender<Message>,
    input: mpsc::Receiver<Message>,
) -> thread::JoinHandle<()> {
    let builder = match config.class {
        Type::SlackChannel => slack::build,
        Type::TermChannel => term::build,
    };

    let seed = Seed {
        name,
        config,
        input,
        output,
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
    pub output: mpsc::Sender<Message>,
    pub input: mpsc::Receiver<Message>,
}

pub trait Channel {
    fn receiver(&self) -> &mpsc::Receiver<Message>;

    fn send_reply(&mut self, r: Reply);

    fn catch_replies(&mut self) -> ReplyResponse {
        let mut did_send = false;

        loop {
            match self.receiver().try_recv() {
                Ok(Message::Hangup) => return ReplyResponse::Hangup,
                Ok(Message::Reply(reply)) => {
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
