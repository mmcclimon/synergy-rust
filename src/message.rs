use crate::user::User;

#[derive(Debug)]
pub enum ChannelEvent {
    Message(ChannelMessage),
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

#[derive(Debug)]
pub enum ReactorEvent {
    Message(ReactorMessage),
}

#[derive(Debug)]
pub struct ReactorMessage {
    pub text: String,
    pub is_public: bool,
    pub was_targeted: bool,
    pub from_address: String,
    pub conversation_address: String,
    pub origin: String,
    pub user: Option<User>,
}
