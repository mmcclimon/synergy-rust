use std::thread;

use crate::message::Event;
use crate::reactor::{Core, Handler, Reactor, Seed};

pub struct Echo {
    core: Core<Dispatch>,
}

pub enum Dispatch {
    HandleEcho,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Echo {
    let core = Core {
        name: seed.name.clone(),
        output: seed.output,
        input: seed.input,
        handlers: vec![Handler {
            require_targeted: true,
            predicate: |e| e.text.starts_with("echo"),
            will_respond: true,
            key: Dispatch::HandleEcho,
        }],
    };

    Echo { core }
}

impl Reactor for Echo {
    type Dispatcher = Dispatch;

    fn core(&self) -> &Core<Dispatch> {
        &self.core
    }

    fn dispatch(&self, key: &Dispatch, event: &Event) {
        match key {
            Dispatch::HandleEcho => self.handle_echo(&event),
        };
    }
}

impl Echo {
    pub fn handle_echo(&self, event: &Event) {
        let who = match &event.user {
            Some(u) => &u.username,
            None => "someone",
        };

        let text = format!("I heard {} say {}", who, event.text);
        self.reply_to(&event, &text);
    }
}
