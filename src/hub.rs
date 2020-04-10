use crate::channel;

pub struct Hub {
    name: String,
    channels: Vec<Box<dyn channel::Channel>>,
}

pub fn new(name: &str) -> Hub {
    // let slack = channel::slack::foo();
    let mut hub = Hub {
        name: name.to_string(),
        channels: vec![],
    };

    hub.channels.push(channel::slack::new());

    hub
}

impl Hub {
    pub fn run(&mut self) {
        info!("running things from hub named {}", self.name);
        for c in &mut self.channels {
            c.start();
        }
    }
}
