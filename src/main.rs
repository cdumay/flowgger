extern crate capnp;
extern crate chrono;
extern crate clap;
#[cfg(feature = "coroutines")]
extern crate coio;
extern crate flate2;
#[cfg(feature = "kafka")]
extern crate rdkafka;
extern crate openssl;
extern crate rand;
extern crate redis;
extern crate serde_json;
extern crate toml;
extern crate env_logger;
extern crate futures;

#[macro_use]
extern crate log;


mod flowgger;

pub use crate::flowgger::record_capnp;

use clap::{App, Arg};

const DEFAULT_CONFIG_FILE: &'static str = "flowgger.toml";
const FLOWGGER_VERSION_STRING: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init();
    let matches = App::new("Flowgger")
        .version(FLOWGGER_VERSION_STRING)
        .about("A fast, simple and lightweight data collector")
        .arg(
            Arg::with_name("config_file")
                .help("Configuration file")
                .value_name("FILE")
                .index(1),
        ).get_matches();
    let config_file = matches
        .value_of("config_file")
        .unwrap_or(DEFAULT_CONFIG_FILE);
    info!("Flowgger {}", FLOWGGER_VERSION_STRING);
    flowgger::start(config_file)
}
