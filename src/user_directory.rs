use std::cell::RefCell;
use std::rc::{Rc, Weak};

use rusqlite::{types::Value, NO_PARAMS};

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

    // TODO: this should return a result.
    pub fn load_users(&self) {
        let env = self.env.borrow().upgrade();
        if env.is_none() {
            warn!("db disappeared out from under us!");
            return;
        }

        info!("TODO: load users");
    }
}
