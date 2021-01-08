use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches};
use crate::app::Server;
use crate::client::Client;
use crate::tasks::TaskClient;
use crate::error::UnwrapOrExit;
use crate::schema::Topic;
use crate::config;


pub fn cli() -> ArgMatches<'static> {
    let app = App::new("task-streamer")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .help("Override config file path")
                .long("config")
                .short("c")
                .takes_value(true)
                .required(false)
                .global(true),
        )
        .subcommand(
            App::new("server")
                .about("start the server")
                .arg(
                    Arg::with_name("port")
                        .help("The port to bind to")
                        .long("port")
                        .short("p")
                        .default_value("8128")
                        .required(false),
                )
                .arg(
                    Arg::with_name("bind")
                        .help("The address to bind to")
                        .long("bind")
                        .short("b")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .required(false),
                )
                .arg(
                    Arg::with_name("api_key")
                        .help("Key client needs to provide for auth to POST endpoint")
                        .long("key")
                        .short("k")
                        .env("TS_API_KEY")
                        .hide_env_values(true)
                        .required(false),
                ),
        )
        .subcommand(
            App::new("client")
                .about("interact with task-streamer server")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .arg(
                    Arg::with_name("api_key")
                        .help("API key")
                        .long("key")
                        .short("k")
                        .env("TS_API_KEY")
                        .hide_env_values(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("server")
                        .help("Server to post tasks to")
                        .long("server")
                        .short("s")
                        .takes_value(true)
                        .required(false),
                )
                .subcommand(
                    App::new("update").about("push tasks to a server").arg(
                        Arg::with_name("filter")
                            .help("The filter to use for listing tasks")
                            .long("filter")
                            .short("f")
                            .default_value("status:pending")
                            .required(false),
                    ),
                )
                .subcommand(
                    App::new("topic")
                        .about("set the topic on the server")
                        .arg(
                            Arg::with_name("title")
                                .help("Use this title")
                                .index(1)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("description")
                                .help("Use this description")
                                .long("description")
                                .short("d")
                                .takes_value(true)
                                .required(false),
                        ),
                ),
        );

    app.clone().get_matches()
}

pub async fn run() -> std::io::Result<()> {
    let matches = cli();
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
