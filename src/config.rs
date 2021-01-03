use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;

use clap::ArgMatches;
use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub client: Client,
}

#[derive(Debug, Default, Deserialize)]
pub struct Server {
    pub port: Option<String>,
    pub bind: Option<Vec<String>>,
    pub api_key: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Client {
    pub server: Option<String>,
    pub filter: Option<String>,
    pub api_key: Option<String>,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Result<Self> {
        let mut config = ::config::Config::new();

        if matches.is_present("config") {
            let path = Path::new(OsStr::new(matches.value_of("config").unwrap()));
            if path.exists() {
                config.merge(::config::File::with_name(path.to_str().unwrap()))?;
            }
        } else {
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
        }

        let mut config: Self = config.try_into()?;

        Config::process_server_options(&mut config, matches);
        Config::process_client_options(&mut config, matches);

        Ok(config)
    }

    fn process_server_options(config: &mut Config, matches: &ArgMatches) {
        let port = matches.value_of("port").unwrap_or("8128");

        if matches.occurrences_of("port") > 0 || config.server.port.is_none() {
            config.server.port = Some(port.to_string());
        }

        if matches.is_present("bind") || config.server.bind.is_none() {
            let mut interfaces: HashSet<&str> = vec!["127.0.0.1"].into_iter().collect();

            if matches.is_present("bind") {
                interfaces = matches.values_of("bind").unwrap().collect();
            }

            config.server.bind = Some(interfaces.into_iter().map(|i| i.to_string()).collect());
        }
    }

    pub fn process_client_options(config: &mut Config, matches: &ArgMatches) {
        let filter = matches.value_of("filter").unwrap_or("status:pending");
        if matches.occurrences_of("filter") > 0 || config.client.filter.is_none() {
            config.client.filter = Some(filter.to_string());
        }

        if matches.is_present("server") {
            config.client.server = Some(matches.value_of("server").unwrap().to_string());
        }

        if matches.is_present("api_key") {
            config.client.api_key = Some(matches.value_of("api_key").unwrap().to_string());
        }
    }
}
