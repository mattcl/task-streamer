use std::collections::HashSet;

use crate::app::Server;
use crate::tasks::TaskClient;

use crate::error::UnwrapOrExit;

mod app;
mod cli;
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
            let client = TaskClient::new(&config.filter.clone().unwrap())
                .unwrap_or_exit("Could not create task client");
            Server::start(&config, client).await
        }
        _ => Ok(()),
    }
}
