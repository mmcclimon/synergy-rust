use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::io::ErrorKind::WouldBlock;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::hub::ChannelSeed;
use crate::message::{ChannelEvent, ChannelMessage, ChannelReply};

type Websocket = tungstenite::protocol::WebSocket<tungstenite::client::AutoStream>;

pub struct Slack {
    pub name: String,
    api_token: String,
    our_name: RefCell<Option<String>>,
    our_id: RefCell<Option<String>>,
    event_tx: mpsc::Sender<ChannelEvent>,
    reply_rx: mpsc::Receiver<ChannelReply>,
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

#[derive(Serialize, Debug)]
struct OutgoingMessage {
    #[serde(rename = "type")]
    kind: String,
    channel: String,
    text: String,
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

pub fn new(seed: ChannelSeed) -> Slack {
    let api_token = &seed.config.extra["api_token"]
        .as_str()
        .expect("no api token in config!");

    Slack {
        name: seed.name.clone(),
        api_token: api_token.to_string(),
        our_id: RefCell::new(None),
        our_name: RefCell::new(None),
        event_tx: seed.event_handle,
        reply_rx: seed.reply_handle,
    }
}

pub fn start(seed: ChannelSeed) -> (String, thread::JoinHandle<()>) {
    let name = seed.name.clone();

    let handle = thread::spawn(move || {
        let channel = self::new(seed);
        channel.start();
    });

    (name, handle)
}

impl Slack {
    fn start(&self) -> ! {
        let mut ws = self.get_websocket().expect("Error connecting to slack!");

        loop {
            loop {
                match self.reply_rx.try_recv() {
                    Ok(ChannelReply::Message(reply)) => {
                        let to_send = OutgoingMessage {
                            kind: "message".to_string(),
                            text: reply.text,
                            channel: reply.conversation_address,
                        };

                        let text = serde_json::to_string(&to_send).unwrap();

                        debug!("writing message: {:?}", to_send);
                        ws.write_message(tungstenite::Message::Text(text)).unwrap();
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("hub hung up on us?");
                    }
                }
            }

            let message = match ws.read_message() {
                Ok(m) => m,
                Err(tungstenite::error::Error::Io(e)) => {
                    if e.kind() == WouldBlock {
                        continue;
                    }

                    info!("IO error reading from websocket: {:?}", e);
                    continue;
                }
                Err(e) => {
                    info!("error reading from websocket: {:?}", e);
                    continue;
                }
            };

            let raw_event = match self.process_ws_message(message) {
                Some(raw) => raw,
                None => continue,
            };

            let msg = self.message_from_raw(raw_event);

            self.event_tx.send(msg).unwrap();
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

        let (mut websocket, _resp) = tungstenite::client::connect(data.url)?;

        info!("connected to slack");

        debug!("setting slack websocket to non-blocking...");
        if let tungstenite::stream::Stream::Tls(stream) = websocket.get_mut() {
            stream
                .get_mut()
                .set_read_timeout(Some(Duration::from_millis(50)))
                .unwrap();
        }

        Ok(websocket)
    }

    fn process_ws_message(&self, message: tungstenite::Message) -> Option<RawEvent> {
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
