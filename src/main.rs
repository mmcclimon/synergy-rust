#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate getopts;

mod channel;
mod config;
mod environment;
mod hub;
mod logger;
mod message;
mod reactor;
mod user;
mod user_directory;

use std::env;
use std::process;

use getopts::Options;

fn main() {
    logger::init();

    // set up options (TODO: use clap, I think.)
    let mut opt = Options::new();
    opt.optflag("", "no-connect", "just boot up, do not connect to slack");
    opt.optopt("c", "config", "config file to use", "FILE");
    opt.optflag("h", "help", "show help and exit");

    let args: Vec<String> = env::args().skip(1).collect();
    let matches = match opt.parse(&args) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("help") {
        let prog_name = env::args().next().unwrap();
        let brief = format!("Usage: {} [options]", prog_name);
        print!("{}", opt.usage(&brief));
        process::exit(0);
    }

    let config = config::new("config.toml");
    let mut hub = hub::new();

    if matches.opt_present("no-connect") {
        info!("exiting early because --no-connect was passed");
        return;
    }

    hub.run(config);
}
