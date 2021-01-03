use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches};

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
