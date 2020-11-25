use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches};

pub fn cli() -> ArgMatches<'static> {
    let app = App::new("task-streamer")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::ColoredHelp)
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
                    Arg::with_name("filter")
                        .help("The filter to use for listing tasks")
                        .long("filter")
                        .short("f")
                        .default_value("status:pending")
                        .required(false),
                ),
        );

    app.clone().get_matches()
}
