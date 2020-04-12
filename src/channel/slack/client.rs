use reqwest::Url;
use serde::Deserialize;
use std::cell::{Cell, RefCell};
use std::error::Error;
use std::fmt;
use std::sync::mpsc;

type Websocket = tungstenite::protocol::WebSocket<tungstenite::client::AutoStream>;

#[derive(Debug)]
pub struct Client {
    api_key: String,
    our_name: RefCell<Option<String>>,
    our_id: RefCell<Option<String>>,
    connected: Cell<bool>,
    is_ready: bool,
    // eventually: more things
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

pub fn new(api_token: String) -> Client {
    let c = Client {
        api_key: api_token.to_string(),
        our_name: RefCell::new(None),
        our_id: RefCell::new(None),
        connected: Cell::new(false),
        is_ready: false,
    };

    return c;
}

#[derive(Debug)]
struct SlackInternalError(String);

impl fmt::Display for SlackInternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error while talking to slack: {}", self.0)
    }
}

impl Error for SlackInternalError {}

impl Client {
    fn connect(&self) -> Result<Websocket, Box<dyn Error>> {
        // using blocking here because I think I'm going to do the concurrent
        // stuff a different way.
        let mut url = Url::parse("https://slack.com/api/rtm.connect")?;
        url.query_pairs_mut().append_pair("token", &self.api_key);

        let client = reqwest::blocking::Client::new();

        #[derive(Deserialize, Debug)]
        struct Me {
            id: Option<String>,
            name: Option<String>,
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

        self.our_name.replace(data.me.name);
        self.our_id.replace(data.me.id);

        let (websocket, _resp) = tungstenite::client::connect(data.url)?;

        self.connected.set(true);

        info!("connected to slack");

        Ok(websocket)
    }

    pub fn listen(&self, tx: mpsc::Sender<RawEvent>) -> ! {
        let mut ws = self.connect().expect("Could not connect to slack!");

        loop {
            let message = match ws.read_message() {
                Ok(m) => m,
                Err(e) => {
                    info!("error reading from websocket: {:?}", e);
                    continue;
                }
            };

            let frame = match message {
                tungstenite::Message::Text(ref s) => s,
                tungstenite::Message::Close(_) => {
                    info!("got close message; figure out what to do here");
                    continue;
                }
                // ignore everything else (ping/pong/binary)
                _ => continue,
            };

            let event: RawEvent = match serde_json::from_str(frame) {
                Ok(re) => re,
                Err(e) => {
                    trace!("error derializing frame {}: {}", frame, e);
                    continue;
                }
            };

            // debug!("got event {:?}", event);
            tx.send(event).unwrap();
        }
    }
}
