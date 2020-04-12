use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use rusqlite::{types::Value, NO_PARAMS};

use crate::environment::Environment;
use crate::user::User;

#[derive(Debug)]
pub struct Directory {
    pub env: RefCell<Weak<Environment>>,
    pub users: RefCell<HashMap<String, User>>,
}

impl Directory {
    pub fn new() -> Rc<Directory> {
        let dir = Rc::new(Directory {
            env: RefCell::new(Weak::new()),
            users: RefCell::new(HashMap::new()),
        });

        dir
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

        fn bool_from(val: Value) -> bool {
            match val {
                Value::Integer(n) => n != 0,
                _ => false,
            }
        }

        let users = stmt.query_map(NO_PARAMS, |row| {
            Ok(User {
                username: row.get_unwrap(0),
                lp_id: match row.get_unwrap(1) {
                    Value::Null => None,
                    Value::Text(s) => Some(s),
                    _ => {
                        debug!("weird value for lp_id!");
                        None
                    }
                },
                is_master: bool_from(row.get_unwrap(2)),
                is_virtual: bool_from(row.get_unwrap(3)),
                is_deleted: bool_from(row.get_unwrap(4)),
                identities: RefCell::new(HashMap::new()),
            })
        });

        // fill up our directory
        for user in users.unwrap() {
            let u = user.unwrap();
            let name = u.username.clone();
            self.users.borrow_mut().insert(name, u);
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
}
