use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use toml::value::Value;

use crate::hub::ChannelSeed;
use crate::message::{ChannelEvent, ChannelMessage, ChannelReply};

pub struct Term {
    pub name: String,
    from_addr: String,
    default_public_reply_addr: String,
    event_tx: mpsc::Sender<ChannelEvent>,
    reply_rx: mpsc::Receiver<ChannelReply>,
}

pub fn start(seed: ChannelSeed) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(seed);
        channel.start();
    });

    (name, handle)
}

pub fn new(seed: ChannelSeed) -> Term {
    let from = match seed.config.extra.get("from_address") {
        Some(Value::String(s)) => s.as_str(),
        Some(_) => "sysop",
        None => "sysop",
    };

    let reply_addr = match seed.config.extra.get("public_reply_addr") {
        Some(Value::String(s)) => s.as_str(),
        Some(_) => "#public",
        None => "#public",
    };

    Term {
        name: seed.name.clone(),
        event_tx: seed.event_handle,
        reply_rx: seed.reply_handle,
        from_addr: from.to_string(),
        default_public_reply_addr: reply_addr.to_string(),
    }
}

impl Term {
    fn start(&self) {
        // we need to kick off a thread for stdin so that we can read from it
        // non-blocking
        let (tx, stdin_rx) = mpsc::channel();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        });

        let mut stdout = io::stdout();
        let mut need_prompt = true;

        loop {
            loop {
                // flush the queue
                match self.reply_rx.try_recv() {
                    Ok(ChannelReply::Message(reply)) => {
                        let indented = reply.text.replace("\n", "\n  ");
                        println!(
                            ">> {}!{} |\n  {}",
                            &self.name, &reply.conversation_address, indented
                        );

                        need_prompt = true;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("hub hung up on us?");
                    }
                }
            }

            if need_prompt {
                print!("synergy> ");
                stdout.flush().unwrap();
                need_prompt = false;
            }

            let text = match stdin_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(s) => s.trim().to_string(),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("stdin hung up on us?");
                }
            };

            // TODO: figure out how to deal with EOF and send SIGINT to parent.
            if text.is_empty() {
                continue;
            }

            let msg = ChannelEvent::Message(ChannelMessage {
                // TODO: fill these in properly
                text: text,
                is_public: false,
                was_targeted: true,
                from_address: self.from_addr.clone(),
                conversation_address: self.default_public_reply_addr.clone(),
                origin: self.name.clone(),
            });

            self.event_tx.send(msg).unwrap();
        }
    }
}