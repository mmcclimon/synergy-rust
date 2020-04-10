pub struct Client {
    api_key: String,
    our_name: String,
    our_id: String,
    connected: bool,
    is_ready: bool,
    // eventually: more things
}

pub fn new() -> Client {
    let api_token =
        option_env!("SLACK_API_TOKEN").expect("Must have SLACK_API_TOKEN in environment!");

    let c = Client {
        api_key: api_token.to_string(),
        our_name: "".to_string(),
        our_id: "".to_string(),
        connected: false,
        is_ready: false,
    };
    return c;
}
