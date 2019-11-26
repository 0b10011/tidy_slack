extern crate reqwest;

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use log::{info, LevelFilter};
use num_format::{Locale, ToFormattedString};
use std::time::Instant;
use std::{fs, u64};
use std::error::Error;
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

    if let Some(cmd) = options.subcommand_name() {
        match cmd {
            "ls" => ls(),
            _ => panic!("Unsupported command: {}", cmd),
        }
    };

    info!(
        "Command completed in {}.{}s",
        now.elapsed().as_secs().to_formatted_string(&Locale::en),
        now.elapsed().subsec_millis()
    );
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ChannelsKind {
    Channels(Channels),
    Error(ChannelsError)
}

#[derive(Deserialize, Debug)]
struct ChannelsError {
    ok: bool,
    error: String,
}
impl std::fmt::Display for ChannelsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
impl Error for ChannelsError {
}

#[derive(Deserialize, Debug)]
struct Channels {
    ok: bool,
    warning: Option<String>,
    channels: Vec<Channel>,
    response_metadata: Metadata,
}

#[derive(Deserialize, Debug)]
struct Channel {
    id: String,
    name: String,
    is_channel: bool,
    is_group: bool,
    is_im: bool,
    created: u64,
    is_archived: bool,
    is_general: bool,
    unlinked: u64,
    name_normalized: String,
    // is_read_only: bool,
    is_shared: bool,
    parent_conversation: Option<String>,
    creator: String,
    is_ext_shared: bool,
    is_org_shared: bool,
    shared_team_ids: Vec<String>, // Not in documentation, but shows up in results
    pending_shared: Vec<String>, // I believe this should always be an empty array?
    pending_connected_team_ids: Vec<String>, // Not in documentation, but shows up in results
    is_pending_ext_shared: bool,
    is_member: bool,
    is_private: bool,
    is_mpim: bool,
    last_read: Option<String>,
    is_open: Option<bool>,
    topic: Topic,
    purpose: Purpose,
    previous_names: Option<Vec<String>>,
    num_members: Option<u64>,
    priority: Option<u64>,
    // locale: String
}

#[derive(Deserialize, Debug)]
struct Topic {
    value: Option<String>,
    creator: Option<String>,
    last_set: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Purpose {
    value: Option<String>,
    creator: Option<String>,
    last_set: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    next_cursor: String,
}

fn get_token() -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string("TOKEN")?.parse::<String>()?.trim().to_string())
}

fn get_conversations() -> Result<Vec<Channel>, Box<dyn Error>> {
    let mut response = Client::new()
        .get("https://slack.com/api/conversations.list")
        .query(&[
            ("exclude_archived", "false"),
            ("limit", "1000"),
            ("types", "public_channel,private_channel,mpim")
        ])
        .header("Authorization", get_token()?)
        .send()?;

    let string = response.text()?;

    println!("Text: {}", string);

    let result = serde_json::from_str::<ChannelsKind>(&string);

    match result? {
        ChannelsKind::Error(error) => Err(error)?,
        ChannelsKind::Channels(channels) => Ok(channels.channels),
    }
}

fn ls() {
    println!("Retrieving conversations...");
    let conversations = get_conversations().unwrap();
    println!("Available conversations:");
    for conversation in conversations {
        let conversation_type = if conversation.is_channel {
            "channel"
        } else if conversation.is_group {
            "group"
        } else if conversation.is_im {
            "im"
        } else {
            "unknown"
        };
        println!("- {} ({}: {})", conversation.id, conversation_type, conversation.name);
    }
}
