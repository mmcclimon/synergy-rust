use std::thread;

use chrono::{FixedOffset, Timelike, Utc};
use chrono_tz::Tz;

use crate::message::Event;
use crate::reactor::{Core, Handler, Reactor, Seed};

pub struct Clox {
    core: Core<Dispatch>,
}

pub enum Dispatch {
    HandleClox,
}

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reactor = self::new(seed);
        reactor.start();
    })
}

pub fn new(seed: Seed) -> Clox {
    let core = Core {
        name: seed.name.clone(),
        reply_tx: seed.reply_handle,
        event_rx: seed.event_handle,
        handlers: vec![Handler {
            predicate: |event| event.text.starts_with("clox"),
            require_targeted: true,
            key: Dispatch::HandleClox,
        }],
    };

    Clox { core }
}

impl Reactor for Clox {
    type Dispatcher = Dispatch;

    fn core(&self) -> &Core<Dispatch> {
        &self.core
    }

    fn dispatch(&self, key: &Dispatch, event: &Event) {
        match key {
            Dispatch::HandleClox => self.handle_clox(&event),
        };
    }
}

impl Clox {
    fn handle_clox(&self, event: &Event) {
        let tzs = vec![
            ("🇺🇸", "America/New_York"),
            ("🇺🇳", "Etc/UTC"),
            ("🇦🇹", "Europe/Vienna"),
            ("🇮🇳", "Asia/Kolkata"),
            ("🇦🇺", "Australia/Melbourne"),
        ];

        let now = Utc::now();
        debug!("{:?}", now.timezone());

        let sit = now.with_timezone(&FixedOffset::east(3600));
        let beats = ((sit.second() + sit.minute() * 60 + sit.hour() * 3600) as f64 / 86.4) as u32;

        let mut text = format!(
            "In Internet Time™ it's {}@{:?}. That's...",
            sit.format("%F"),
            beats,
        );

        for (abbrev, tz_name) in &tzs {
            let tz: Tz = tz_name.parse().unwrap();
            text.push_str(&format!(
                "\n{} {}",
                abbrev,
                now.with_timezone(&tz).format("%F %H:%M")
            ));
        }

        self.reply_to(&event, &text);
    }
}
