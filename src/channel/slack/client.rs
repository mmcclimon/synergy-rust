use reqwest::Url;
use serde::Deserialize;
use std::error::Error;
use std::fmt;

pub struct Client {
    api_key: String,
    our_name: Option<String>,
    our_id: Option<String>,
    connected: bool,
    is_ready: bool,
    // eventually: more things
}

pub fn new() -> Client {
    let api_token =
        option_env!("SLACK_API_TOKEN").expect("Must have SLACK_API_TOKEN in environment!");

    let c = Client {
        api_key: api_token.to_string(),
        our_name: None,
        our_id: None,
        connected: false,
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
    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
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

        println!("{:?}", data);

        if !data.ok {
            return Err(Box::new(SlackInternalError(
                "bad data from connect".to_string(),
            )));
        }

        self.our_name = data.me.name;
        self.our_id = data.me.id;

        Ok(())
    }
}
