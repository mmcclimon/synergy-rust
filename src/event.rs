// use crate::channel::Channel;
use crate::user::User;

#[derive(Debug)]
pub enum EventType {
    Message,
}

#[derive(Debug)]
pub struct Event {
    pub kind: EventType,
    pub from_user: Option<User>,
    pub text: String,
    pub is_public: bool,
    pub was_targeted: bool,
    pub from_address: String,
    pub conversation_address: String,

    // In perl synergy, this is a ref to the channel, but I don't think that's
    // going to work, because here we're doing things across threads and
    // channels are not (necessarily?) safe to share between threads. If we just
    // stick a name in here, the hub can look them up by name such that they can
    // reply to it.
    pub from_channel_name: String,
}
