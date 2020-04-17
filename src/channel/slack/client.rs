use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::io::ErrorKind::WouldBlock;
use std::time::Duration;

use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::message::Reply;

type Websocket = tungstenite::protocol::WebSocket<tungstenite::client::AutoStream>;

// boxes up our websocket
pub struct Client {
    ws: RefCell<Option<Websocket>>,
}

// This is a raw message event, and only matches messages, because that's the
// only thing we care about. Other things will try to deserialize to this and
// not be able to, in which case we'll just ignore it. That's not the "proper"
// way to do it, but gets us up and running.
#[derive(Deserialize, Debug)]
pub struct RawEvent {
    ts: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub subtype: Option<String>,
    pub channel: String,
    pub text: String,
    pub user: String,
    bot_id: Option<String>,
}

// guts

#[derive(Deserialize, Debug)]
pub struct SlackIdentity {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Debug)]
struct OutgoingMessage {
    #[serde(rename = "type")]
    kind: String,
    channel: String,
    text: String,
}

#[derive(Debug)]
struct SlackInternalError(String);

impl Error for SlackInternalError {}

impl fmt::Display for SlackInternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while talking to slack: {}", self.0)
    }
}

pub fn new() -> Client {
    Client {
        ws: RefCell::new(None),
    }
}

impl Client {
    pub fn connect(&self, api_token: &str) -> SlackIdentity {
        let (ws, me) = get_websocket(api_token).expect("Error connecting to slack!");

        self.ws.replace(Some(ws));

        me
    }

    pub fn send(&self, reply: Reply) {
        let to_send = OutgoingMessage {
            kind: "message".to_string(),
            text: reply.text,
            channel: reply.conversation_address,
        };

        let text = serde_json::to_string(&to_send).unwrap();

        debug!("writing message: {:?}", to_send);
        self.ws
            .borrow_mut()
            .as_mut()
            .unwrap()
            .write_message(tungstenite::Message::Text(text))
            .unwrap();
    }

    pub fn recv(&self) -> Option<RawEvent> {
        let message = match self.ws.borrow_mut().as_mut().unwrap().read_message() {
            Ok(m) => m,
            Err(tungstenite::error::Error::Io(e)) => {
                if e.kind() == WouldBlock {
                    return None;
                }

                info!("IO error reading from websocket: {:?}", e);
                return None;
            }
            Err(e) => {
                info!("error reading from websocket: {:?}", e);
                return None;
            }
        };

        return self.process_ws_message(message);
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
}

fn get_websocket(api_token: &str) -> Result<(Websocket, SlackIdentity), Box<dyn Error>> {
    // using blocking here because I think I'm going to do the concurrent
    // stuff a different way.
    let mut url = Url::parse("https://slack.com/api/rtm.connect")?;
    url.query_pairs_mut().append_pair("token", api_token);

    let client = reqwest::blocking::Client::new();

    #[derive(Deserialize, Debug)]
    struct ConnectResp {
        ok: bool,
        url: String,
        #[serde(rename = "self")]
        me: SlackIdentity,
    }

    let data: ConnectResp = client.get(url).send()?.json()?;

    if !data.ok {
        return Err(Box::new(SlackInternalError(
            "bad data from connect".to_string(),
        )));
    }

    let me = data.me;

    let (mut websocket, _resp) = tungstenite::client::connect(data.url)?;

    info!("connected to slack");

    debug!("setting slack websocket to non-blocking...");
    if let tungstenite::stream::Stream::Tls(stream) = websocket.get_mut() {
        stream
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
    }

    Ok((websocket, me))
}
