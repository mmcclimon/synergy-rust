// use crate::channel::Channel;
use crate::environment::Environment;
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

impl Event {
    pub fn ensure_complete(&mut self, env: &Environment) {
        let users = env.user_directory.users.borrow();
        let user = users
            .values()
            .filter(|u| {
                u.identities
                    .borrow()
                    .iter()
                    .filter(|(name, val)| {
                        // why are these doubly referenced? dunno, really, but
                        // the compiler was happy with this!
                        self.from_address == **val && self.from_channel_name == **name
                    })
                    .next()
                    .is_some()
            })
            .next();

        if let Some(u) = user {
            // cloning is gross here, but I don't want to futz with references
            // and lifetimes at the moment
            self.from_user = Some(u.clone());
        }
    }
}
