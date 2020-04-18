use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use rusqlite::NO_PARAMS;

use crate::environment::Environment;
use crate::message::Event;
use crate::user::User;

#[derive(Debug)]
pub struct Directory {
    pub env: RefCell<Weak<Environment>>,
    pub users: RefCell<HashMap<String, User>>,
    identities: RefCell<HashMap<String, HashMap<String, String>>>,
}

// identities: {
//   channel/name: {
//      "addr": "username"
//   }
// }

impl Directory {
    pub fn new() -> Arc<Directory> {
        Arc::new(Directory {
            env: RefCell::new(Weak::new()),
            users: RefCell::new(HashMap::new()),
            identities: RefCell::new(HashMap::new()),
        })
    }

    // TODO: this should return a result.
    pub fn load_users(&self) {
        let env = self.env.borrow().upgrade();
        if env.is_none() {
            warn!("db disappeared out from under us!");
            return;
        }

        let db = &env.unwrap().db;
        let mut stmt = db.prepare("SELECT * FROM users").unwrap();

        let iter = stmt
            .query_map(NO_PARAMS, |row| Ok(User::from(row)))
            .unwrap();

        // fill up our directory
        for maybe_user in iter {
            let user = maybe_user.unwrap();
            let name = user.username.clone();
            self.users.borrow_mut().insert(name, user);
        }

        self.load_identities(&db);
    }

    // we pass db here to avoid having to upgrade() it again.
    fn load_identities(&self, db: &rusqlite::Connection) {
        let mut stmt = db
            .prepare("select username, identity_name, identity_value from user_identities")
            .unwrap();

        let identities_iter = stmt.query_map(NO_PARAMS, |row| {
            let username: String = row.get_unwrap(0);
            let name: String = row.get_unwrap(1);
            let val: String = row.get_unwrap(2);

            Ok((username, name, val))
        });

        let mut identities = self.identities.borrow_mut();

        for identity in identities_iter.unwrap() {
            let (who, channel_name, addr) = identity.unwrap();

            let munged_name = format!("channel/{}", channel_name);

            let for_channel = if identities.contains_key(&munged_name) {
                identities.get_mut(&munged_name).unwrap()
            } else {
                let key = munged_name.clone();
                identities.insert(munged_name, HashMap::new());
                identities.get_mut(&key).unwrap()
            };

            for_channel.insert(addr, who);
        }
    }

    pub fn resolve_user(&self, event: &Event) -> Option<User> {
        let idents = self.identities.borrow();

        let channel_identities = match idents.get(&event.origin) {
            Some(i) => i,
            None => return None,
        };

        let username = channel_identities.get(&event.from_address);
        match username {
            Some(name) => Some(self.users.borrow().get(name).unwrap().clone()),
            None => None,
        }
    }
}
