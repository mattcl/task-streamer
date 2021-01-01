use clap;
use crate::app::Server;
use crate::client::Client;
use crate::tasks::TaskClient;

use crate::error::{TSError, UnwrapOrExit};

mod app;
mod cli;
mod client;
mod config;
mod error;
mod session;
mod tasks;

/*
use actix_files as fs;
*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::cli();
    match matches.subcommand() {
        ("server", Some(server_matches)) => {
            let config =
                config::Config::new(&server_matches).unwrap_or_exit("Could not load config file");
            Server::start(config).await
        }
        ("push", Some(push_matches)) => {
            let config =
                config::Config::new(&push_matches).unwrap_or_exit("Could not load config file");
            let task_client = TaskClient::new(&config.client.filter.clone().unwrap())
                .unwrap_or_exit("Could not create task client");

            if config.client.server.is_none() {
                let err = clap::Error::with_description(
                    &"Server must be specified either in config or via parameter",
                    clap::ErrorKind::InvalidValue,
                );
                err.exit()
            }

            if config.client.api_key.is_none() {
                let err = clap::Error::with_description(
                    &"Api key must be specified either in config or via parameter",
                    clap::ErrorKind::InvalidValue,
                );
                err.exit()
            }

            let client = Client::new(config);
            client.push_tasks(&task_client).await.unwrap_or_exit("Could not post tasks");
            Ok(())
        }
        _ => Ok(()),
    }
}


