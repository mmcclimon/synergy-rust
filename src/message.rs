use std::sync::Arc;
use uuid::Uuid;

use crate::user::User;

#[derive(Debug)]
pub enum Message<T> {
    Text(Arc<T>),
    Hangup,
    Ack(String, bool),
}

// FIXME all these names are terrible.

#[derive(Debug)]
pub struct Event {
    pub text: String,
    pub is_public: bool,
    pub was_targeted: bool,
    pub from_address: String,
    pub conversation_address: String,
    pub origin: String,
    pub user: Option<User>,
    pub id: String,
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
    pub fn new_id() -> String {
        format!("{}", Uuid::new_v4())
    }

    pub fn reply(&self, text: &str, origin: &str) -> Message<Reply> {
        Message::Text(Arc::new(Reply {
            text: text.to_string(),
            from_address: self.from_address.clone(),
            conversation_address: self.conversation_address.clone(),
            origin: origin.to_string(),
            destination: self.origin.clone(),
        }))
    }

    pub fn dupe(&self) -> Self {
        Event {
            text: self.text.clone(),
            is_public: self.is_public,
            was_targeted: self.was_targeted,
            from_address: self.from_address.clone(),
            conversation_address: self.conversation_address.clone(),
            origin: self.origin.clone(),
            user: self.user.clone(),
            id: self.id.clone(),
        }
    }
}
