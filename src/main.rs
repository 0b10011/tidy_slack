extern crate reqwest;

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use log::{info, LevelFilter};
use num_format::{Locale, ToFormattedString};
use std::time::Instant;
use std::{u64};
use reqwest::Error;
use reqwest::Client;
use serde::{Deserialize};

fn main() {
    let now = Instant::now();

    let options = App::new("Tidy Slack")
        .version(crate_version!())
        .author("Brandon Frohs <brandon@19.codes>")
        .about("Deletes messages from slack.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        // Subcommands
        .subcommand(SubCommand::with_name("ls")
            .about("List conversations the authenticated user can access.")
        )
        // Verbosity level
        .arg(
            Arg::with_name("silent")
                .short("s")
                .long("silent")
                .conflicts_with_all(&["quiet", "verbose"])
                .help("Silences all output."),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .multiple(true)
                .long("quiet")
                .conflicts_with("verbose")
                .help("Shows less detail. -q shows less detail, -qq shows least detail, -qqq is equivalent to --silent."),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .long("verbose")
                .help("Shows more detail. -v shows more detail, -vv shows most detail."),
        )
        .get_matches();

    // Default log level is Info
    // --silent switches level to Off
    // -v, --verbose increases level to debug
    // -vv increases level to Trace
    // -q, --quiet decreases level to Error
    // -qq decreases level to Off
    let filter: LevelFilter = if options.is_present("silent") {
        LevelFilter::Off
    } else {
        match options.occurrences_of("verbose") {
            0 => match options.occurrences_of("quiet") {
                3..=u64::MAX => LevelFilter::Off,
                2 => LevelFilter::Error,
                1 => LevelFilter::Warn,
                0 => LevelFilter::Info, // Default
            },
            1 => LevelFilter::Debug,
            2..=u64::MAX => LevelFilter::Trace,
        }
    };

    env_logger::Builder::from_default_env()
        .filter(Some(module_path!()), filter)
        .init();

    // if let Some(cmd) = options.subcommand_name() {
    //     match cmd {
    //         "ls" => ls(),
    //         _ => ls(),// Err(format!("Unsupported command: {}", cmd)),
    //     }
    // };
    match ls() {
        Ok(()) => info!("It worked!"),
        _ => info!("It didn't work :("),
    }

    info!(
        "Command completed in {}.{}s",
        now.elapsed().as_secs().to_formatted_string(&Locale::en),
        now.elapsed().subsec_millis()
    );
}

#[derive(Deserialize, Debug)]
enum ChannelsKind {
    // Channels,
    ChannelsError
}

#[derive(Deserialize, Debug)]
struct ChannelsError {
    ok: bool,
    error: String,
}

#[derive(Deserialize, Debug)]
struct Channels {
    ok: bool,
    channels: Vec<Channel>,
}

#[derive(Deserialize, Debug)]
struct Channel {
    id: String,
    name: String,
}

fn ls() -> Result<(), Error> {
    let request_url = "https://slack.com/api/users.conversations";

    let client = Client::new();

    let mut response = client
        .get(request_url)
        .send()?;

    let string = response.text()?;
    println!("{:?}", serde_json::from_str::<ChannelsKind>(&string));
    let json: ChannelsKind = response.json()?;
    println!("{:?}", json);

    Ok(())
}
