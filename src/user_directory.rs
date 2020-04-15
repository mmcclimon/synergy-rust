use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

use rusqlite::NO_PARAMS;

use crate::environment::Environment;
use crate::message::ChannelMessage;
use crate::user::User;

#[derive(Debug)]
pub struct Directory {
    pub env: RefCell<Weak<Environment>>,
    pub users: RefCell<HashMap<String, User>>,
}

impl Directory {
    pub fn new() -> Arc<Directory> {
        Arc::new(Directory {
            env: RefCell::new(Weak::new()),
            users: RefCell::new(HashMap::new()),
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

        for identity in identities_iter.unwrap() {
            let (who, name, val) = identity.unwrap();
            let users = self.users.borrow_mut();
            let user = users.get(&who).unwrap();
            user.add_identity(name, val);
        }
    }

    pub fn resolve_user(&self, event: &ChannelMessage) -> Option<User> {
        let users = self.users.borrow();
        let user = users
            .values()
            .filter(|u| {
                u.identities
                    .borrow()
                    .iter()
                    .filter(|(name, val)| event.from_address == **val && event.origin == **name)
                    .next()
                    .is_some()
            })
            .next();

        match user {
            // cloning is gross here, but I don't want to futz with references
            // and lifetimes at the moment
            Some(u) => Some(u.clone()),
            None => None,
        }
    }
}
