pub mod echo;

use serde::Deserialize;

use crate::config::ComponentConfig;
use crate::event::Event;

// known reactors
#[derive(Deserialize, Debug)]
pub enum Type {
    EchoReactor,
}

pub struct Handler {
    pub name: String,
    pub matcher: Box<dyn Fn(&Event) -> bool>,
    pub handler: Box<dyn Fn() -> ()>,
}

type ReactorConfig = ComponentConfig<Type>;

pub trait Reactor {
    fn name(&self) -> String;

    fn react_to(&self, event: &Event) -> ();
}
