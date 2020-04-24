use std::io::{self, Write};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use colorful::Colorful;
use toml::value::Value;

use crate::channel::{Channel, ReplyResponse, Seed};
use crate::message::{Event, Message, Reply};

pub struct Term {
    pub name: String,
    from_addr: String,
    default_public_reply_addr: String,
    to_hub: mpsc::Sender<Message>,
    from_hub: mpsc::Receiver<Message>,
}

enum TermValue {
    Text(String),
    EOF,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut channel = self::new(seed);
        channel.start();
    })
}

pub fn new(seed: Seed) -> Term {
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
        to_hub: seed.output,
        from_hub: seed.input,
        from_addr: from.to_string(),
        default_public_reply_addr: reply_addr.to_string(),
    }
}

impl Channel for Term {
    fn receiver(&self) -> &mpsc::Receiver<Message> {
        &self.from_hub
    }

    fn send_reply(&mut self, reply: Reply) {
        let indented = reply.text.replace("\n", "\n  ");
        let text = format!(
            ">> {}!{} |\n  {}",
            &self.name, &reply.conversation_address, indented,
        );

        println!("{}", text.magenta());
    }
}

impl Term {
    fn start(&mut self) {
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
            match self.catch_replies() {
                ReplyResponse::Sent => need_prompt = true,
                ReplyResponse::Hangup => break,
                _ => (),
            };

            if need_prompt {
                print!("{}", "rustergy> ".cyan());
                stdout.flush().unwrap();
                need_prompt = false;
            }

            let value = match stdin_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(s) => {
                    // 0 bytes here is EOF, blank line is just '\n'
                    if s.is_empty() {
                        TermValue::EOF
                    } else {
                        TermValue::Text(s.trim().to_string())
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("stdin hung up on us?");
                }
            };

            let text = match value {
                TermValue::EOF => {
                    println!(); // so log line doesn't show up on prompt line
                    self.to_hub.send(Message::Hangup).unwrap();
                    break;
                }
                TermValue::Text(s) => s,
            };

            if text.is_empty() {
                continue;
            }

            let msg = Message::Event(Arc::new(Event {
                // TODO: fill these in properly
                text,
                is_public: false,
                was_targeted: true,
                from_address: self.from_addr.clone(),
                conversation_address: self.default_public_reply_addr.clone(),
                origin: self.name.clone(),
                user: None,
                id: Event::new_id(),
            }));

            self.to_hub.send(msg).unwrap();
        }
    }
}
