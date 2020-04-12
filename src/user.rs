use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub lp_id: Option<String>,
    pub is_master: bool,
    pub is_virtual: bool,
    pub is_deleted: bool,
    pub identities: RefCell<HashMap<String, String>>,
}

impl User {
    pub fn add_identity(&self, name: String, value: String) {
        self.identities.borrow_mut().insert(name, value);
    }
}

// This is so that the code in the user directory is a little nicer. It assumes
// that the columns have not been renamed from their database defaults. We
// could, maybe, do better and introspect the row itself, but also, it hardly
// matters, because this is only ever going to be called from one place. Mostly
// this was an excuse to write a From implementation.
impl From<&rusqlite::Row<'_>> for User {
    fn from(row: &rusqlite::Row) -> Self {
        use rusqlite::types::Value;

        fn bool_from(val: Value) -> bool {
            match val {
                Value::Integer(n) => n != 0,
                _ => false,
            }
        }

        let username = row.get_unwrap("username");

        let lp_id = match row.get_unwrap("lp_id") {
            Value::Text(s) => Some(s),
            _ => None,
        };

        let is_master = bool_from(row.get_unwrap("is_master"));
        let is_virtual = bool_from(row.get_unwrap("is_virtual"));
        let is_deleted = bool_from(row.get_unwrap("is_deleted"));

        return User {
            username: username,
            lp_id: lp_id,
            is_master: is_master,
            is_virtual: is_virtual,
            is_deleted: is_deleted,
            identities: RefCell::new(HashMap::new()),
        };
    }
}
