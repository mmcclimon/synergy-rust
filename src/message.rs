use crate::user::User;

#[derive(Debug)]
pub enum Message<T> {
    Text(T),
    Hangup,
}

// FIXME all these names are terrible.

#[derive(Debug, Clone)]
pub struct Event {
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

impl Event {
    pub fn reply(&self, text: &str, origin: &str) -> Message<Reply> {
        Message::Text(Reply {
            text: text.to_string(),
            from_address: self.from_address.clone(),
            conversation_address: self.conversation_address.clone(),
            origin: origin.to_string(),
            destination: self.origin.clone(),
        })
    }
}
