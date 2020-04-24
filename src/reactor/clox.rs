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
        output: seed.output,
        input: seed.input,
        handlers: vec![Handler {
            predicate: |event| event.text.starts_with("clox"),
            require_targeted: true,
            will_respond: true,
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

        let sit = now.with_timezone(&FixedOffset::east(3600));
        let beats = ((sit.second() + sit.minute() * 60 + sit.hour() * 3600) as f64 / 86.4) as u32;

        let mut text = format!(
            "In Internet Time™ it's {}@{:?}. That's...",
            sit.format("%F"),
            beats,
        );

        // TODO
        let user_time = now.with_timezone(&"America/New_York".parse::<Tz>().unwrap());

        for (abbrev, tz_name) in &tzs {
            let tz: Tz = tz_name.parse().unwrap();
            let local = now.with_timezone(&tz);

            let day_delta = local
                .naive_local()
                .date()
                .signed_duration_since(user_time.naive_local().date())
                .num_days();

            let pretty_day = match day_delta {
                -2 => String::from("the day before yesterday"),
                -1 => String::from("yesterday"),
                0 => String::from("today"),
                1 => String::from("tomorrow"),
                2 => String::from("the day after tomorrow"),
                _ => format!("{}", local.format("%F")),
            };

            text.push_str(&format!(
                "\n{} {} at {}",
                abbrev,
                pretty_day,
                local.format("%H:%M"),
            ));
        }

        self.reply_to(&event, &text);
    }
}
