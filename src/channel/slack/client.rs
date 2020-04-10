use reqwest::Url;
use serde::Deserialize;
use std::error::Error;

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

        Ok(())
    }
}
