use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::environment::Environment;

#[derive(Debug)]
pub struct Directory {
    pub env: RefCell<Weak<Environment>>,
}

impl Directory {
    pub fn new() -> Rc<Directory> {
        let dir = Rc::new(Directory {
            env: RefCell::new(Weak::new()),
        });

        dir
    }

    pub fn load_users(&self) {
        let _db = match self.env.borrow().upgrade() {
            Some(ref env) => &env.db,
            None => {
                warn!("db disappeared out from under us!");
                return;
            }
        };

        info!("TODO: load users");
    }
}
