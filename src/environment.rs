use std::fmt;
use std::sync::Arc;

use rusqlite::{Connection, NO_PARAMS};

use crate::config::Config;
use crate::message::Event;
use crate::user::User;
use crate::user_directory::Directory;

pub struct Environment {
    pub db: Connection,
    pub user_directory: Arc<Directory>,
}

pub fn new(config: &Config) -> Arc<Environment> {
    let conn = Connection::open(&config.state_dbfile).expect("Could not open dbfile!");

    // make the user directory first, with an empty env (internally). Once we
    // have constructed ourselves with a strong ref to the directory, we'll
    // give the directory a weak ref of ourself.
    let env = Arc::new(Environment {
        db: conn,
        user_directory: Directory::new(),
    });

    // I am a little surprised this works.
    let ud = Arc::clone(&env.user_directory);
    *ud.env.borrow_mut() = Arc::downgrade(&env);

    env.maybe_create_state_tables();
    ud.load_users();

    env
}

// we must do this because sqlite::Connection is not Debug
impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("db", &"[ sqlite connection ]")
            .field("user_directory", &self.user_directory)
            .finish()
    }
}

impl Environment {
    pub fn resolve_user(&self, event: &Event) -> Option<User> {
        self.user_directory.resolve_user(&event)
    }

    fn maybe_create_state_tables(&self) {
        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS synergy_state (\n  \
                    reactor_name TEXT PRIMARY KEY,\n  \
                    stored_at INTEGER NOT NULL,\n  \
                    json TEXT NOT NULL\n\
                    );",
                NO_PARAMS,
            )
            .unwrap();

        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS users (\n  \
                    username TEXT PRIMARY KEY,\n  \
                    lp_id TEXT,\n  \
                    is_master INTEGER DEFAULT 0,\n  \
                    is_virtual INTEGER DEFAULT 0,\n  \
                    is_deleted INTEGER DEFAULT 0\n\
                );",
                NO_PARAMS,
            )
            .unwrap();

        self.db
            .execute(
                "CREATE TABLE IF NOT EXISTS user_identities (\n  \
                    id INTEGER PRIMARY KEY,\n  \
                    username TEXT NOT NULL,\n  \
                    identity_name TEXT NOT NULL,\n  \
                    identity_value TEXT NOT NULL,\n  \
                    FOREIGN KEY (username) REFERENCES users(username) ON DELETE CASCADE,\n  \
                    CONSTRAINT constraint_username_identity UNIQUE (username, identity_name),\n  \
                    UNIQUE (identity_name, identity_value)\n\
                );",
                NO_PARAMS,
            )
            .unwrap();
    }
}
