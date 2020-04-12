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
