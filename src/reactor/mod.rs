pub mod echo;

use serde::Deserialize;

use crate::config::ComponentConfig;
use crate::event::Event;

// known reactors
#[derive(Deserialize, Debug)]
pub enum Type {
    EchoReactor,
}

type Handler = fn(Event) -> ();

type ReactorConfig = ComponentConfig<Type>;

pub trait Reactor {
    fn name(&self) -> String;

    fn handlers_matching(&self, event: &Event) -> Vec<Handler>;
}
