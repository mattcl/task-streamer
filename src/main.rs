use std::collections::HashSet;
use std::sync::Mutex;

use actix::prelude::*;
use crate::app::Server;

mod app;
mod cli;
mod error;
mod tasks;

/*
use actix_files as fs;
*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = cli::cli();
    match matches.subcommand() {
        ("server", Some(server_matches)) => {
            let port = server_matches.value_of("port").unwrap();

            let mut interfaces: HashSet<&str> = vec!["127.0.0.1"].into_iter().collect();

            if server_matches.is_present("bind") {
                interfaces = server_matches.values_of("bind").unwrap().collect();
            }

            Server::start(&port, &interfaces).await
        }
        _ => Ok(()),
    }
}

