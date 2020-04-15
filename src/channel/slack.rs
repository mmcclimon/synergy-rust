use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::sync::mpsc;
use std::thread;

use reqwest::Url;
use serde::Deserialize;

use crate::channel::ChannelConfig;
use crate::hub::Seed;
use crate::message::{ChannelEvent, ChannelMessage};

type Websocket = tungstenite::protocol::WebSocket<tungstenite::client::AutoStream>;
type TungsteniteResult = Result<tungstenite::Message, tungstenite::Error>;

pub struct Slack {
    pub name: String,
    api_token: String,
    our_name: RefCell<Option<String>>,
    our_id: RefCell<Option<String>>,
    // hub: RefCell<Weak<Hub>>,
}

// This is a raw message event, and only matches messages, because that's the
// only thing we care about. Other things will try to deserialize to this and
// not be able to, in which case we'll just ignore it. That's not the "proper"
// way to do it, but gets us up and running.
#[derive(Deserialize, Debug)]
struct RawEvent {
    ts: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: Option<String>,
    pub channel: String,
    pub text: String,
    pub user: String,
    bot_id: Option<String>,
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

pub fn new(seed: &Seed<ChannelConfig>) -> Slack {
    let api_token = &seed.config.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    Slack {
        name: seed.name.clone(),
        api_token: api_token.to_string(),
        our_id: RefCell::new(None),
        our_name: RefCell::new(None),
        // hub: RefCell::new(hub),
    }
}

pub fn start(mut seed: Seed<ChannelConfig>) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(&seed);
        channel.start(seed.event_handle);
    });

    (name, handle)
}

impl Slack {
    fn start(&self, events_channel: mpsc::Sender<ChannelEvent>) -> ! {
        info!("starting slack channel {}", self.name);

        let mut ws = self.get_websocket().expect("Error connecting to slack!");

        // TODO here, we also need a channel to send events

        loop {
            let raw_event = match self.process_ws_message(ws.read_message()) {
                Some(raw) => raw,
                None => continue,
            };

            let msg = self.message_from_raw(raw_event);

            events_channel.send(msg).unwrap();
        }
    }

    fn get_websocket(&self) -> Result<Websocket, Box<dyn Error>> {
        // using blocking here because I think I'm going to do the concurrent
        // stuff a different way.
        let mut url = Url::parse("https://slack.com/api/rtm.connect")?;
        url.query_pairs_mut().append_pair("token", &self.api_token);

        let client = reqwest::blocking::Client::new();

        #[derive(Deserialize, Debug)]
        struct Me {
            id: String,
            name: String,
        }

        #[derive(Deserialize, Debug)]
        struct ConnectResp {
            ok: bool,
            url: String,
            #[serde(rename = "self")]
            me: Me,
        }

        let data: ConnectResp = client.get(url).send()?.json()?;

        if !data.ok {
            return Err(Box::new(SlackInternalError(
                "bad data from connect".to_string(),
            )));
        }

        self.our_name.replace(Some(data.me.name));
        self.our_id.replace(Some(data.me.id));

        let (websocket, _resp) = tungstenite::client::connect(data.url)?;

        info!("connected to slack");

        Ok(websocket)
    }

    fn process_ws_message(&self, raw: TungsteniteResult) -> Option<RawEvent> {
        let message = match raw {
            Ok(m) => m,
            Err(e) => {
                info!("error reading from websocket: {:?}", e);
                return None;
            }
        };

        let frame = match message {
            tungstenite::Message::Text(ref s) => s,
            tungstenite::Message::Close(_) => {
                info!("got close message; figure out what to do here");
                return None;
            }
            // ignore everything else (ping/pong/binary)
            _ => return None,
        };

        let event: RawEvent = match serde_json::from_str(frame) {
            Ok(re) => re,
            Err(e) => {
                trace!("error derializing frame {}: {}", frame, e);
                return None;
            }
        };

        // debug!("got event {:?}", event);
        return Some(event);
    }

    fn message_from_raw(&self, raw: RawEvent) -> ChannelEvent {
        ChannelEvent::Message(ChannelMessage {
            // TODO: fill these in properly
            text: raw.text,
            is_public: false,
            was_targeted: true,
            from_address: raw.user,
            conversation_address: raw.channel,
            origin: self.name.clone(),
        })
    }
}
