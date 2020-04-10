pub struct Slack {}

pub fn new() -> Box<Slack> {
    eprintln!("slack::new()");
    return Box::new(Slack {});
}

impl super::Channel for Slack {
    fn start(&self) {
        println!("starting slack channel");
    }
}
