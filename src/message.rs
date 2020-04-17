use crate::user::User;

// FIXME all these names are terrible.

#[derive(Debug)]
pub enum ChannelEvent {
    Message(ChannelMessage),
    Hangup,
}

#[derive(Debug)]
pub enum ChannelReply {
    Message(Reply),
}

#[derive(Debug)]
pub struct ChannelMessage {
    pub text: String,
    pub is_public: bool,
    pub was_targeted: bool,
    pub from_address: String,
    pub conversation_address: String,
    pub origin: String,
}

#[derive(Debug, Clone)]
pub enum ReactorEvent {
    Message(ReactorMessage),
}

#[derive(Debug)]
pub enum ReactorReply {
    Message(Reply),
}

#[derive(Debug, Clone)]
pub struct ReactorMessage {
    pub text: String,
    pub is_public: bool,
    pub was_targeted: bool,
    pub from_address: String,
    pub conversation_address: String,
    pub origin: String,
    pub user: Option<User>,
}

// I think eventually, I want some sort of unique identifier per [channel]event,
// and then set reply_to here, so that we can keep track of things that don't
// get replies (maybe).
#[derive(Debug, Clone)]
pub struct Reply {
    pub text: String,
    pub from_address: String,
    pub conversation_address: String,
    pub origin: String,
    pub destination: String,
}

impl ReactorMessage {
    pub fn reply(&self, text: &str, origin: &str) -> ReactorReply {
        ReactorReply::Message(Reply {
            text: text.to_string(),
            from_address: self.from_address.clone(),
            conversation_address: self.conversation_address.clone(),
            origin: origin.to_string(),
            destination: self.origin.clone(),
        })
    }
}

impl From<ReactorReply> for ChannelReply {
    fn from(r: ReactorReply) -> Self {
        match r {
            ReactorReply::Message(reply) => ChannelReply::Message(reply.clone()),
        }
    }
}
