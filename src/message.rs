use crate::event::Event;

#[derive(Debug)]
pub enum ChannelEvent {
    Message(Event),
}
