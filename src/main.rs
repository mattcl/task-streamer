use crate::app::Server;
use crate::client::Client;
use crate::tasks::TaskClient;
use clap;

use crate::error::UnwrapOrExit;
use crate::schema::Topic;

mod app;
mod cli;
mod client;
mod config;
mod error;
mod schema;
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

            if config.server.api_key.is_none() {
                let err = clap::Error::with_description(
                    &"Api key must be specified either in config or via parameter",
                    clap::ErrorKind::InvalidValue,
                );
                err.exit()
            }

            Server::start(config).await
        }
        ("client", Some(client_matches)) => {
            let mut config =
                config::Config::new(&client_matches).unwrap_or_exit("Could not load config file");

            match client_matches.subcommand() {
                ("update", Some(update_matches)) => {
                    // we have to pick up the `filter` flag
                    config::Config::process_client_options(&mut config, &update_matches);
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
                    client
                        .push_tasks(&task_client)
                        .await
                        .unwrap_or_exit("Could not post tasks");
                    Ok(())
                }
                ("topic", Some(topic_matches)) => {
                    let client = Client::new(config);
                    let topic = Topic::new(
                        topic_matches.value_of("title").unwrap().to_string(),
                        topic_matches
                            .value_of("description")
                            .unwrap_or_default()
                            .to_string(),
                    );

                    client
                        .set_topic(topic)
                        .await
                        .unwrap_or_exit("Could not set topic");
                    Ok(())
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
