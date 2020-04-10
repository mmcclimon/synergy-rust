mod client;

pub struct Slack {
    client: client::Client,
}

pub fn new() -> Box<Slack> {
    eprintln!("slack::new()");

    let channel = Slack {
        client: client::new(),
    };

    return Box::new(channel);
}

impl super::Channel for Slack {
    fn start(&self) {
        println!("starting slack channel");
    }
}
