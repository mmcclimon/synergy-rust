use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;

use crate::channel;
use crate::reactor;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub state_dbfile: String,

    // work me out later
    pub channels: HashMap<String, ComponentConfig<channel::Type>>,
    pub reactors: HashMap<String, ComponentConfig<reactor::Type>>,
}

#[derive(Deserialize, Debug)]
pub struct ComponentConfig<T> {
    pub class: T,

    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

pub fn new(filename: &str) -> Config {
    let path = Path::new(filename);

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("couldn't open {}: {:?}", filename, e),
    };

    let mut s = String::new();

    if let Err(e) = file.read_to_string(&mut s) {
        panic!("couldn't read {}: {:?}", filename, e);
    };

    let config: Config = match toml::from_str(&s) {
        Ok(c) => c,
        Err(e) => {
            panic!("invalid config file: {}", e);
        }
    };

    config
}
