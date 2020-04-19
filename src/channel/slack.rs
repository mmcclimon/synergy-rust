mod api_client;
mod rtm_client;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

use regex::{Captures, Regex};

use crate::channel::{Channel, ReplyResponse, Seed};
use crate::message::{Event, Message, Reply};
use api_client::ApiClient;
use rtm_client::{RawEvent, RtmClient};

pub struct Slack {
    pub name: String,
    api_token: String,
    rtm_client: RtmClient,
    api_client: ApiClient,
    event_tx: mpsc::Sender<Message<Event>>,
    reply_rx: mpsc::Receiver<Message<Reply>>,

    // cached data
    our_name: Option<String>,
    our_id: Option<String>,
    targeted_re: Regex, // I could use an option here, but.
    users: Option<HashMap<String, String>>,
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

pub fn build(seed: Seed) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut channel = self::new(seed);
        channel.start();
    })
}

pub fn new(seed: Seed) -> Slack {
    let api_token = &seed.config.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    Slack {
        name: seed.name.clone(),
        api_token: api_token.to_string(),
        event_tx: seed.event_handle,
        reply_rx: seed.reply_handle,
        rtm_client: rtm_client::new(),
        api_client: api_client::new(api_token.to_string()),
        our_id: None,
        our_name: None,
        targeted_re: Regex::new("").unwrap(),
        users: None,
    }
}

impl Channel for Slack {
    fn receiver(&self) -> &mpsc::Receiver<Message<Reply>> {
        &self.reply_rx
    }

    fn send_reply(&mut self, reply: Reply) {
        self.rtm_client.send(reply);
    }
}

impl Slack {
    fn start(&mut self) {
        let me = self.rtm_client.connect(&self.api_token);

        self.targeted_re = Regex::new(&format!(r"(?i)^@?{}:?\s*", me.name)).unwrap();
        self.our_name = Some(me.name);
        self.our_id = Some(me.id);

        // this block: maybe it would be better not to do so.
        self.users = self.api_client.load_users();

        loop {
            match self.catch_replies() {
                ReplyResponse::Hangup => break,
                _ => (),
            };

            let raw_event = match self.rtm_client.recv() {
                Some(raw) => raw,
                None => continue,
            };

            let event = match self.event_from_raw(raw_event) {
                Some(e) => e,
                None => continue,
            };

            self.event_tx.send(Message::Text(event)).unwrap();
        }
    }

    fn event_from_raw(&self, raw: RawEvent) -> Option<Event> {
        let text = self.decode_slack_formatting(raw.text);

        let mut was_targeted = self.targeted_re.is_match(&text);

        // anything in DM is targeted
        if raw.channel.starts_with("D") {
            was_targeted = true;
        }

        let is_public = raw.channel.starts_with("C");

        Some(Event {
            text,
            is_public,
            was_targeted,
            from_address: raw.user,
            conversation_address: raw.channel,
            origin: self.name.clone(),
            user: None,
        })
    }

    fn decode_slack_formatting(&self, text: String) -> String {
        lazy_static! {
            static ref USERNAME_RE: Regex = Regex::new(r"<@(U[A-Z0-9]+)>").unwrap();
            static ref CHANNEL_RE: Regex = Regex::new(r"<#[CD](?:[A-Z0-9]+)\|(.*?)>").unwrap();
            static ref MAILTO_RE: Regex = Regex::new(r"<mailto:\S+?\|([^>]+)>").unwrap();
            static ref URL_RE: Regex = Regex::new(r"<[^|]+\|([^>]+)>").unwrap();
        };

        // TODO: here, swap slack userids for slack usernames (which we need to
        // look up)
        let subbed_users = USERNAME_RE.replace_all(&text, |caps: &Captures| {
            let name = self.username_for(&caps[1]);
            format!("@{}", name)
        });

        let subbed_channels = CHANNEL_RE.replace_all(&subbed_users, "#$1");
        let subbed_mailto = MAILTO_RE.replace_all(&subbed_channels, "$1");
        let subbed_url = URL_RE.replace_all(&subbed_mailto, "$1");

        // switch to a string to do the rest.
        let mut out = subbed_url.replace("<", "");
        out = out.replace(">", "");

        // re-encode html
        out = out.replace("&lt;", "<");
        out = out.replace("&gt;", ">");
        out = out.replace("&amp;", "&");

        String::from(out)
    }

    fn username_for(&self, slackid: &str) -> String {
        // TODO stop unwrap()ing so much
        self.users
            .as_ref()
            .unwrap()
            .get(slackid)
            .unwrap()
            .to_string()
    }
}
