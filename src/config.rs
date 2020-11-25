use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use clap::ArgMatches;
use serde::Deserialize;

use crate::error::Result;

#[derive(Deserialize)]
pub struct Config {
    pub port: Option<String>,
    pub bind: Option<Vec<String>>,
    pub filter: Option<String>,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Result<Self> {
        let mut config = ::config::Config::new();

        if let Some(home) = dirs::home_dir() {
            let global_config = home.join(Path::new("task-streamer.toml"));

            if global_config.exists() {
                if let Some(path) = global_config.to_str() {
                    config.merge(::config::File::with_name(path))?;
                }
            }
        }

        if Path::new(OsStr::new("task-streamer.toml")).exists() {
            config.merge(::config::File::with_name("task-streamer"))?;
        }

        let mut config: Self = config.try_into()?;

        let port = matches.value_of("port").unwrap();
        let filter = matches.value_of("filter").unwrap();

        if matches.occurrences_of("port") > 0 || config.port.is_none() {
            config.port = Some(port.to_string());
        }

        if matches.occurrences_of("filter") > 0 || config.filter.is_none() {
            config.filter = Some(filter.to_string());
        }

        if matches.is_present("bind") || config.bind.is_none() {
            let mut interfaces: HashSet<&str> = vec!["127.0.0.1"].into_iter().collect();

            if matches.is_present("bind") {
                interfaces = matches.values_of("bind").unwrap().collect();
            }

            config.bind = Some(interfaces.into_iter().map(|i| i.to_string()).collect());
        }

        Ok(config)
    }
}
